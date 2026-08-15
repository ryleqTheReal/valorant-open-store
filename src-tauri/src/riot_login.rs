// ======= START RIOT LOGIN =======

use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

use crate::config;

#[derive(thiserror::Error, Debug)]
pub enum RiotLoginError {
    #[error("failed to build riot authorize url: {0}")]
    UrlBuildFailed(url::ParseError),
    #[error("failed to create riot login webview: {0}")]
    WebviewCreateFailed(tauri::Error),
    #[error("riot login window closed before auth completed")]
    WindowClosed,
    #[error("riot login oneshot channel dropped unexpectedly")]
    ChannelFailed,
}

pub async fn do_login(app: &AppHandle, pkce_challenge: &str) -> Result<String, RiotLoginError> {
    let mut authorize_url =
        url::Url::parse(config::RIOT_AUTHORIZE_URL).map_err(RiotLoginError::UrlBuildFailed)?;
    authorize_url
        .query_pairs_mut()
        .append_pair("client_id", config::RIOT_CLIENT_ID)
        .append_pair("redirect_uri", config::RIOT_REDIRECT_URI)
        .append_pair("response_type", "code")
        .append_pair("scope", config::RIOT_SCOPE)
        .append_pair("code_challenge", pkce_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("nonce", &Uuid::new_v4().to_string());

    let (tx, rx) = tokio::sync::oneshot::channel::<Option<String>>();
    let tx = Arc::new(Mutex::new(Some(tx)));

    let tx_nav = Arc::clone(&tx);
    let tx_close = Arc::clone(&tx);

    // unique label so concurrent logins don't collide
    let label = format!("riot-login-{}", Uuid::new_v4().simple());

    let app_for_nav = app.clone();
    let label_for_nav = label.clone();

    let win = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(authorize_url))
        .title("Riot Login")
        .inner_size(480.0, 720.0)
        .incognito(true)
        .on_navigation(move |nav_url| {
            if nav_url.as_str().starts_with(config::RIOT_REDIRECT_URI) {
                let code = nav_url
                    .query_pairs()
                    .find(|(k, _)| k == "code")
                    .map(|(_, v)| v.into_owned());
                if let Ok(mut guard) = tx_nav.lock() {
                    if let Some(sender) = guard.take() {
                        let _ = sender.send(code);
                    }
                }

                if let Some(win) = app_for_nav.get_webview_window(&label_for_nav) {
                    let _ = win.close();
                }
                return false;
            }
            true
        })
        .build()
        .map_err(RiotLoginError::WebviewCreateFailed)?;

    win.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Destroyed) {
            if let Ok(mut guard) = tx_close.lock() {
                if let Some(sender) = guard.take() {
                    let _ = sender.send(None);
                }
            }
        }
    });

    match rx.await {
        Ok(Some(code)) => Ok(code),
        Ok(None) => Err(RiotLoginError::WindowClosed),
        Err(_) => Err(RiotLoginError::ChannelFailed),
    }
}

// ======= END RIOT LOGIN =======
