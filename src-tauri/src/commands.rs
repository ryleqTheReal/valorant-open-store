// ======= START COMMANDS =======

use reqwest::Client;
use tauri::{AppHandle, State};

use crate::{
    discord_auth, dpop, riot_auth, riot_login,
    session::Session,
    store_client,
    types::{AccountSummary, AddAccountResponse, DiscordProfile},
    uninstall,
};

#[tauri::command]
pub async fn login_discord(
    app: AppHandle,
    client: State<'_, Client>,
    session: State<'_, Session>,
) -> Result<DiscordProfile, String> {
    let (token_pair, profile) = discord_auth::login(&app, &client)
        .await
        .map_err(|e| e.to_string())?;
    session.set(token_pair, profile.clone());
    Ok(profile)
}

#[tauri::command]
pub async fn add_account(
    app: AppHandle,
    client: State<'_, Client>,
    session: State<'_, Session>,
) -> Result<AddAccountResponse, String> {
    let access_token = session
        .access_token()
        .ok_or_else(|| "not logged in".to_string())?;

    let dpop_key = dpop::generate_keypair();
    let pkce = riot_auth::generate_pkce();
    let code = riot_login::do_login(&app, &pkce.challenge)
        .await
        .map_err(|e| e.to_string())?;
    let tokens = riot_auth::exchange_code(&client, &dpop_key, &code, &pkce.verifier)
        .await
        .map_err(|e| e.to_string())?;
    store_client::add_account(&client, &access_token, dpop_key, tokens)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_accounts(
    client: State<'_, Client>,
    session: State<'_, Session>,
) -> Result<Vec<AccountSummary>, String> {
    let access_token = session
        .access_token()
        .ok_or_else(|| "not logged in".to_string())?;
    store_client::list_accounts(&client, &access_token)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unlink_account(
    client: State<'_, Client>,
    session: State<'_, Session>,
    puuid: String,
) -> Result<(), String> {
    let access_token = session
        .access_token()
        .ok_or_else(|| "not logged in".to_string())?;
    store_client::delete_account(&client, &access_token, &puuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn logout(session: State<'_, Session>) -> Result<(), String> {
    session.logout();
    Ok(())
}

#[tauri::command]
pub async fn uninstall(app: AppHandle) -> Result<(), String> {
    uninstall::run().map_err(|e| e.to_string())?;
    app.exit(0);
    Ok(())
}

// ======= END COMMANDS =======
