// ======= START RIOT AUTH =======

use base64ct::{Base64UrlUnpadded, Encoding};
use rand_core::RngCore;
use reqwest::Client;
use sha2::{Digest, Sha256};

use crate::{
    config,
    dpop::{self, DpopError},
    types::{DpopKey, TokenSet},
};

#[derive(thiserror::Error, Debug)]
pub enum RiotAuthError {
    #[error("riot token exchange request failed: {0}")]
    RequestFailed(reqwest::Error),
    #[error("riot token exchange {status}: {body}")]
    BadStatus { status: u16, body: String },
    #[error("riot token exchange dpop nonce loop exhausted")]
    NonceLoopExhausted,
    #[error("riot token exchange response missing '{field}'")]
    MissingField { field: &'static str },
    #[error("riot entitlements request failed: {0}")]
    EntitlementsRequestFailed(reqwest::Error),
    #[error("riot entitlements {status}: {body}")]
    EntitlementsBadStatus { status: u16, body: String },
    #[error("riot entitlements response has no token field")]
    MissingEntitlements,
    #[error("dpop error: {0}")]
    Dpop(#[from] DpopError),
    #[error("response json parse failed: {0}")]
    JsonParseFailed(reqwest::Error),
}

pub struct PkceParams {
    pub verifier: String,
    pub challenge: String,
}

pub fn generate_pkce() -> PkceParams {
    let mut bytes = [0u8; 32];
    rand_core::OsRng.fill_bytes(&mut bytes);
    let verifier = Base64UrlUnpadded::encode_string(&bytes);
    let hash = Sha256::digest(verifier.as_bytes());
    let challenge = Base64UrlUnpadded::encode_string(&hash);
    PkceParams {
        verifier,
        challenge,
    }
}

pub async fn exchange_code(
    client: &Client,
    key: &DpopKey,
    code: &str,
    pkce_verifier: &str,
) -> Result<TokenSet, RiotAuthError> {
    let (access_token, refresh_token, id_token, expires_in) =
        dpop_code_post(client, key, code, pkce_verifier).await?;

    let entitlements_token = fetch_entitlements(client, &access_token).await?;
    let expires_at = chrono::Utc::now().timestamp() + expires_in;

    Ok(TokenSet {
        access_token,
        refresh_token,
        id_token,
        entitlements_token,
        expires_at,
    })
}

async fn dpop_code_post(
    client: &Client,
    key: &DpopKey,
    code: &str,
    pkce_verifier: &str,
) -> Result<(String, String, String, i64), RiotAuthError> {
    let mut nonce: Option<String> = None;

    for attempt in 0u8..=1 {
        let proof =
            dpop::create_proof(key, "POST", config::RIOT_TOKEN_URL, None, nonce.as_deref())?;

        let res = client
            .post(config::RIOT_TOKEN_URL)
            .header("User-Agent", config::RIOT_UA)
            .header("Accept", "application/json")
            .header("DPoP", proof)
            .form(&[
                ("client_id", config::RIOT_CLIENT_ID),
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", config::RIOT_REDIRECT_URI),
                ("code_verifier", pkce_verifier),
                ("riot_prm", config::RIOT_PRM),
            ])
            .send()
            .await
            .map_err(RiotAuthError::RequestFailed)?;

        let status = res.status();

        if status == 400 {
            let dpop_nonce = res
                .headers()
                .get("dpop-nonce")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            let body: serde_json::Value = res.json().await.unwrap_or_default();
            if attempt == 0 && body.get("error").and_then(|e| e.as_str()) == Some("use_dpop_nonce")
            {
                nonce = dpop_nonce;
                continue;
            }
            return Err(RiotAuthError::BadStatus {
                status: 400,
                body: body.to_string(),
            });
        }

        if !status.is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(RiotAuthError::BadStatus {
                status: status.as_u16(),
                body,
            });
        }

        let body: serde_json::Value = res.json().await.map_err(RiotAuthError::JsonParseFailed)?;
        let access_token = body["access_token"]
            .as_str()
            .ok_or(RiotAuthError::MissingField {
                field: "access_token",
            })?
            .to_string();
        let refresh_token = body["refresh_token"]
            .as_str()
            .ok_or(RiotAuthError::MissingField {
                field: "refresh_token",
            })?
            .to_string();
        let id_token = body["id_token"].as_str().unwrap_or("").to_string();
        let expires_in = body["expires_in"].as_i64().unwrap_or(3600);

        return Ok((access_token, refresh_token, id_token, expires_in));
    }

    Err(RiotAuthError::NonceLoopExhausted)
}

pub async fn fetch_entitlements(
    client: &Client,
    access_token: &str,
) -> Result<String, RiotAuthError> {
    let res = client
        .post(config::RIOT_ENTITLEMENTS_URL)
        .header("User-Agent", config::RIOT_UA)
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer {access_token}"))
        .json(&serde_json::json!({ "urn": "urn:entitlement:%" }))
        .send()
        .await
        .map_err(RiotAuthError::EntitlementsRequestFailed)?;

    if !res.status().is_success() {
        let status = res.status().as_u16();
        let body = res.text().await.unwrap_or_default();
        return Err(RiotAuthError::EntitlementsBadStatus { status, body });
    }

    let body: serde_json::Value = res.json().await.map_err(RiotAuthError::JsonParseFailed)?;
    let token = body["entitlements_token"]
        .as_str()
        .or_else(|| body["token"].as_str())
        .or_else(|| body["entitlement_token"].as_str())
        .ok_or(RiotAuthError::MissingEntitlements)?
        .to_string();

    Ok(token)
}

// ======= END RIOT AUTH =======

// ======= START TESTS =======

#[cfg(test)]
mod tests {
    use super::*;
    use base64ct::{Base64UrlUnpadded, Encoding};
    use sha2::{Digest, Sha256};

    #[test]
    fn pkce_challenge_is_sha256_of_verifier() {
        let params = generate_pkce();
        let verifier_bytes = Base64UrlUnpadded::decode_vec(&params.verifier).unwrap();
        assert_eq!(verifier_bytes.len(), 32);

        let expected_hash = Sha256::digest(params.verifier.as_bytes());
        let expected_challenge = Base64UrlUnpadded::encode_string(&expected_hash);
        assert_eq!(params.challenge, expected_challenge);
    }

    #[test]
    fn pkce_verifier_is_base64url_unpadded() {
        let params = generate_pkce();
        assert!(!params.verifier.contains('='), "verifier must be unpadded");
        assert!(
            !params.challenge.contains('='),
            "challenge must be unpadded"
        );
    }

    #[test]
    fn pkce_two_calls_produce_different_verifiers() {
        let a = generate_pkce();
        let b = generate_pkce();
        assert_ne!(a.verifier, b.verifier);
    }
}

// ======= END TESTS =======
