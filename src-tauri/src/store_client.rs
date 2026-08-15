// ======= START STORE CLIENT =======

use reqwest::Client;

use crate::{
    config,
    types::{AccountSummary, AddAccountRequest, AddAccountResponse, DpopKey, TokenSet},
};

#[derive(thiserror::Error, Debug)]
pub enum StoreClientError {
    #[error("add account request failed: {0}")]
    AddAccountRequestFailed(reqwest::Error),
    #[error("add account {status}: {body}")]
    AddAccountBadStatus { status: u16, body: String },
    #[error("list accounts request failed: {0}")]
    ListAccountsRequestFailed(reqwest::Error),
    #[error("list accounts {status}: {body}")]
    ListAccountsBadStatus { status: u16, body: String },
    #[error("delete account request failed: {0}")]
    DeleteAccountRequestFailed(reqwest::Error),
    #[error("delete account {status}: {body}")]
    DeleteAccountBadStatus { status: u16, body: String },
    #[error("response json parse failed: {0}")]
    JsonParseFailed(reqwest::Error),
}

pub async fn add_account(
    client: &Client,
    access_token: &str,
    dpop_key: DpopKey,
    tokens: TokenSet,
) -> Result<AddAccountResponse, StoreClientError> {
    let body = AddAccountRequest { dpop_key, tokens };
    let res = client
        .post(format!("{}/v1/accounts", config::store()))
        .header("Authorization", format!("Bearer {access_token}"))
        .json(&body)
        .send()
        .await
        .map_err(StoreClientError::AddAccountRequestFailed)?;

    if res.status().as_u16() != 201 {
        let status = res.status().as_u16();
        let body = res.text().await.unwrap_or_default();
        return Err(StoreClientError::AddAccountBadStatus { status, body });
    }

    res.json().await.map_err(StoreClientError::JsonParseFailed)
}

pub async fn list_accounts(
    client: &Client,
    access_token: &str,
) -> Result<Vec<AccountSummary>, StoreClientError> {
    let res = client
        .get(format!("{}/v1/accounts", config::store()))
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await
        .map_err(StoreClientError::ListAccountsRequestFailed)?;

    if !res.status().is_success() {
        let status = res.status().as_u16();
        let body = res.text().await.unwrap_or_default();
        return Err(StoreClientError::ListAccountsBadStatus { status, body });
    }

    let value: serde_json::Value = res
        .json()
        .await
        .map_err(StoreClientError::JsonParseFailed)?;
    let accounts = value["accounts"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    Ok(accounts)
}

pub async fn delete_account(
    client: &Client,
    access_token: &str,
    puuid: &str,
) -> Result<(), StoreClientError> {
    let res = client
        .delete(format!("{}/v1/accounts/{puuid}", config::store()))
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await
        .map_err(StoreClientError::DeleteAccountRequestFailed)?;

    if res.status().as_u16() != 204 {
        let status = res.status().as_u16();
        let body = res.text().await.unwrap_or_default();
        return Err(StoreClientError::DeleteAccountBadStatus { status, body });
    }

    Ok(())
}

// ======= END STORE CLIENT =======
