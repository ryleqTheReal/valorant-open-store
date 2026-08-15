// ======= START DISCORD AUTH =======

use std::time::Duration;

use reqwest::Client;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::{
    config,
    types::{DiscordProfile, WebTokenPair},
};

#[derive(thiserror::Error, Debug)]
pub enum DiscordAuthError {
    #[error("loopback port acquisition failed: {0}")]
    PortFailed(std::io::Error),
    #[error("loopback server bind failed: {0}")]
    BindFailed(String),
    #[error("browser open failed: {0}")]
    BrowserOpenFailed(String),
    #[error("loopback wait timed out after 300s")]
    Timeout,
    #[error("loopback callback receive failed: {0}")]
    RecvFailed(std::io::Error),
    #[error("discord callback missing required field '{field}'")]
    MissingField { field: &'static str },
    #[error("discord auth background task failed: {0}")]
    TaskFailed(String),
    #[error("vw-api login/web request failed: {0}")]
    LoginRequestFailed(reqwest::Error),
    #[error("vw-api login/web {status}: {body}")]
    LoginBadStatus { status: u16, body: String },
    #[error("vw-api login/web json parse failed: {0}")]
    LoginDeserFailed(serde_json::Error),
    #[error("discord profile request failed: {0}")]
    ProfileRequestFailed(reqwest::Error),
    #[error("discord profile {status}: {body}")]
    ProfileBadStatus { status: u16, body: String },
    #[error("discord profile response missing '{field}'")]
    ProfileMissingField { field: &'static str },
    #[error("response json parse failed: {0}")]
    JsonParseFailed(reqwest::Error),
}

pub async fn login(
    app: &AppHandle,
    client: &Client,
) -> Result<(WebTokenPair, DiscordProfile), DiscordAuthError> {
    let tmp = std::net::TcpListener::bind("127.0.0.1:0").map_err(DiscordAuthError::PortFailed)?;
    let port = tmp
        .local_addr()
        .map_err(DiscordAuthError::PortFailed)?
        .port();
    drop(tmp);

    let server = tiny_http::Server::http(format!("127.0.0.1:{port}"))
        .map_err(|e| DiscordAuthError::BindFailed(e.to_string()))?;

    let mut auth_url = url::Url::parse(&format!("{}/v1/auth/discord", config::vw_api()))
        .expect("vw-api base url is a valid url prefix");
    auth_url
        .query_pairs_mut()
        .append_pair("redirect", "true")
        .append_pair(
            "loopback_redirect",
            &format!("http://127.0.0.1:{port}/callback"),
        );

    app.opener()
        .open_url(auth_url.as_str(), None::<&str>)
        .map_err(|e| DiscordAuthError::BrowserOpenFailed(e.to_string()))?;

    let (provider, provider_id, discord_token) = tokio::task::spawn_blocking(
        move || -> Result<(String, String, String), DiscordAuthError> {
            let request = server
                .recv_timeout(Duration::from_secs(300))
                .map_err(DiscordAuthError::RecvFailed)?
                .ok_or(DiscordAuthError::Timeout)?;

            let url_str = format!("http://127.0.0.1{}", request.url());
            let parsed = url::Url::parse(&url_str).map_err(|_| DiscordAuthError::MissingField {
                field: "callback url",
            })?;

            let mut provider = None;
            let mut provider_id = None;
            let mut access_token = None;
            for (k, v) in parsed.query_pairs() {
                match k.as_ref() {
                    "provider" => provider = Some(v.into_owned()),
                    "provider_id" => provider_id = Some(v.into_owned()),
                    "access_token" => access_token = Some(v.into_owned()),
                    _ => {}
                }
            }

            let header =
                tiny_http::Header::from_bytes(b"Location", config::signed_in_url().as_bytes())
                    .expect("static header is valid");
            let response = tiny_http::Response::empty(302).with_header(header);
            let _ = request.respond(response);

            Ok((
                provider.ok_or(DiscordAuthError::MissingField { field: "provider" })?,
                provider_id.ok_or(DiscordAuthError::MissingField {
                    field: "provider_id",
                })?,
                access_token.ok_or(DiscordAuthError::MissingField {
                    field: "access_token",
                })?,
            ))
        },
    )
    .await
    .map_err(|e| DiscordAuthError::TaskFailed(e.to_string()))??;

    let login_res = client
        .post(format!("{}/v1/login/web", config::vw_api()))
        .json(&serde_json::json!({
            "provider": &provider,
            "provider_id": &provider_id,
            "access_token": &discord_token,
        }))
        .send()
        .await
        .map_err(DiscordAuthError::LoginRequestFailed)?;

    let login_status = login_res.status();
    let login_body = login_res.text().await.unwrap_or_default();
    eprintln!("[open-store] login/web status={login_status} body={login_body}");

    if !login_status.is_success() {
        return Err(DiscordAuthError::LoginBadStatus {
            status: login_status.as_u16(),
            body: login_body,
        });
    }

    let web_token: WebTokenPair =
        serde_json::from_str(&login_body).map_err(DiscordAuthError::LoginDeserFailed)?;

    // fetch Discord avatar for the UI
    let profile_res = client
        .get("https://discord.com/api/users/@me")
        .header("Authorization", format!("Bearer {discord_token}"))
        .send()
        .await
        .map_err(DiscordAuthError::ProfileRequestFailed)?;

    if !profile_res.status().is_success() {
        let status = profile_res.status().as_u16();
        let body = profile_res.text().await.unwrap_or_default();
        return Err(DiscordAuthError::ProfileBadStatus { status, body });
    }

    let pj: serde_json::Value = profile_res
        .json()
        .await
        .map_err(DiscordAuthError::JsonParseFailed)?;
    let id = pj["id"]
        .as_str()
        .ok_or(DiscordAuthError::ProfileMissingField { field: "id" })?
        .to_string();
    let username = pj["username"]
        .as_str()
        .ok_or(DiscordAuthError::ProfileMissingField { field: "username" })?
        .to_string();
    let avatar = pj["avatar"].as_str().unwrap_or("");
    let avatar_url = if avatar.is_empty() {
        "https://cdn.discordapp.com/embed/avatars/0.png".to_string()
    } else {
        format!("https://cdn.discordapp.com/avatars/{id}/{avatar}.png")
    };

    Ok((
        web_token,
        DiscordProfile {
            id,
            username,
            avatar_url,
        },
    ))
}

// ======= END DISCORD AUTH =======
