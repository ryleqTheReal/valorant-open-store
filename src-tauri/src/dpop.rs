// ======= START DPOP =======

use base64ct::{Base64UrlUnpadded, Encoding};
use p256::ecdsa::{Signature, SigningKey, signature::Signer};
use rand_core::OsRng;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::types::DpopKey;

#[derive(thiserror::Error, Debug)]
pub enum DpopError {
    #[error("dpop key 'd' base64 decode failed: {0}")]
    KeyDecodeFailed(base64ct::Error),
    #[error("dpop key 'd' must be 32 bytes, got {0}")]
    KeyBadLength(usize),
    #[error("invalid dpop signing key: {0}")]
    InvalidSigningKey(p256::ecdsa::Error),
    #[error("dpop url parse failed: {0}")]
    UrlParseFailed(url::ParseError),
    #[error("dpop json serialization failed: {0}")]
    JsonFailed(serde_json::Error),
}

pub fn generate_keypair() -> DpopKey {
    let signing_key = SigningKey::random(&mut OsRng);
    let d_bytes = signing_key.to_bytes();
    let d = Base64UrlUnpadded::encode_string(&d_bytes[..]);
    let point = signing_key.verifying_key().to_encoded_point(false);
    let x_bytes = point.x().expect("uncompressed point has x");
    let x = Base64UrlUnpadded::encode_string(&x_bytes[..]);
    let y_bytes = point.y().expect("uncompressed point has y");
    let y = Base64UrlUnpadded::encode_string(&y_bytes[..]);
    DpopKey {
        kty: "EC".into(),
        crv: "P-256".into(),
        x,
        y,
        d,
    }
}

pub fn create_proof(
    key: &DpopKey,
    method: &str,
    url: &str,
    access_token: Option<&str>,
    nonce: Option<&str>,
) -> Result<String, DpopError> {
    let d_bytes = Base64UrlUnpadded::decode_vec(&key.d).map_err(DpopError::KeyDecodeFailed)?;
    if d_bytes.len() != 32 {
        return Err(DpopError::KeyBadLength(d_bytes.len()));
    }
    let signing_key =
        SigningKey::from_bytes(d_bytes.as_slice().into()).map_err(DpopError::InvalidSigningKey)?;

    let mut parsed = url::Url::parse(url).map_err(DpopError::UrlParseFailed)?;
    parsed.set_query(None);
    parsed.set_fragment(None);

    let header = serde_json::json!({
        "typ": "dpop+jwt",
        "alg": "ES256",
        "jwk": { "kty": &key.kty, "crv": &key.crv, "x": &key.x, "y": &key.y }
    });

    let mut payload = serde_json::json!({
        "jti": Uuid::new_v4().to_string(),
        "htm": method.to_uppercase(),
        "htu": parsed.to_string(),
        "iat": chrono::Utc::now().timestamp(),
    });

    if let Some(at) = access_token {
        let hash = Sha256::digest(at.as_bytes());
        payload["ath"] = serde_json::json!(Base64UrlUnpadded::encode_string(&hash));
    }
    if let Some(n) = nonce {
        payload["nonce"] = serde_json::json!(n);
    }

    let h = Base64UrlUnpadded::encode_string(
        serde_json::to_string(&header)
            .map_err(DpopError::JsonFailed)?
            .as_bytes(),
    );
    let p = Base64UrlUnpadded::encode_string(
        serde_json::to_string(&payload)
            .map_err(DpopError::JsonFailed)?
            .as_bytes(),
    );
    let signing_input = format!("{h}.{p}");

    let sig: Signature = signing_key.sign(signing_input.as_bytes());
    let sig_b64 = Base64UrlUnpadded::encode_string(sig.to_bytes().as_ref());

    Ok(format!("{signing_input}.{sig_b64}"))
}

// ======= END DPOP =======

// ======= START TESTS =======

#[cfg(test)]
mod tests {
    use super::*;
    use base64ct::{Base64UrlUnpadded, Encoding};

    #[test]
    fn generate_keypair_d_is_32_bytes() {
        let key = generate_keypair();
        let d = Base64UrlUnpadded::decode_vec(&key.d).unwrap();
        assert_eq!(d.len(), 32);
        assert_eq!(key.kty, "EC");
        assert_eq!(key.crv, "P-256");
    }

    #[test]
    fn generate_keypair_x_y_valid_point() {
        let key = generate_keypair();
        let x = Base64UrlUnpadded::decode_vec(&key.x).unwrap();
        let y = Base64UrlUnpadded::decode_vec(&key.y).unwrap();
        assert_eq!(x.len(), 32);
        assert_eq!(y.len(), 32);
    }

    #[test]
    fn generate_keypair_round_trips_through_create_proof() {
        let key = generate_keypair();
        let result = create_proof(&key, "POST", "https://example.com/token", None, None);
        assert!(result.is_ok(), "create_proof failed: {:?}", result.err());
    }

    #[test]
    fn create_proof_three_segments() {
        let key = generate_keypair();
        let token = create_proof(&key, "POST", "https://example.com/token", None, None).unwrap();
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3, "jwt must have 3 dot-separated segments");
    }

    #[test]
    fn create_proof_header_payload_decode() {
        let key = generate_keypair();
        let token =
            create_proof(&key, "GET", "https://example.com/path?q=1#frag", None, None).unwrap();
        let mut parts = token.splitn(3, '.');

        let header_bytes = Base64UrlUnpadded::decode_vec(parts.next().unwrap()).unwrap();
        let header: serde_json::Value = serde_json::from_slice(&header_bytes).unwrap();
        assert_eq!(header["typ"], "dpop+jwt");
        assert_eq!(header["alg"], "ES256");
        assert!(header["jwk"].is_object());

        let payload_bytes = Base64UrlUnpadded::decode_vec(parts.next().unwrap()).unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&payload_bytes).unwrap();
        assert_eq!(payload["htm"], "GET");
        // htu must strip query and fragment
        assert_eq!(payload["htu"], "https://example.com/path");
        assert!(payload["jti"].is_string());
        assert!(payload["iat"].is_i64());
    }

    #[test]
    fn create_proof_signature_is_64_bytes() {
        let key = generate_keypair();
        let token = create_proof(&key, "POST", "https://example.com/", None, None).unwrap();
        let sig_b64 = token.rsplit('.').next().unwrap();
        let sig_bytes = Base64UrlUnpadded::decode_vec(sig_b64).unwrap();
        assert_eq!(sig_bytes.len(), 64);
    }

    #[test]
    fn create_proof_with_access_token_includes_ath() {
        let key = generate_keypair();
        let token = create_proof(
            &key,
            "POST",
            "https://example.com/",
            Some("my-access-token"),
            None,
        )
        .unwrap();
        let payload_b64 = token.split('.').nth(1).unwrap();
        let payload_bytes = Base64UrlUnpadded::decode_vec(payload_b64).unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&payload_bytes).unwrap();
        assert!(payload["ath"].is_string());
    }

    #[test]
    fn create_proof_with_nonce_includes_nonce_field() {
        let key = generate_keypair();
        let token = create_proof(
            &key,
            "POST",
            "https://example.com/",
            None,
            Some("test-nonce"),
        )
        .unwrap();
        let payload_b64 = token.split('.').nth(1).unwrap();
        let payload_bytes = Base64UrlUnpadded::decode_vec(payload_b64).unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&payload_bytes).unwrap();
        assert_eq!(payload["nonce"], "test-nonce");
    }

    #[test]
    fn create_proof_bad_key_returns_error() {
        let mut key = generate_keypair();
        key.d = "not-valid-base64!!!".into();
        assert!(create_proof(&key, "POST", "https://example.com/", None, None).is_err());
    }
}

// ======= END TESTS =======
