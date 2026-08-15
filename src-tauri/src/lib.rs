// ======= START MODULES =======

mod commands;
mod config;
mod discord_auth;
mod dpop;
mod riot_auth;
mod riot_login;
mod session;
mod store_client;
mod types;
mod uninstall;

// ======= END MODULES =======

// ======= START APP =======

pub fn run() {
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .expect("failed to build reqwest client");

    let session = session::Session::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(client)
        .manage(session)
        .invoke_handler(tauri::generate_handler![
            commands::login_discord,
            commands::add_account,
            commands::list_accounts,
            commands::unlink_account,
            commands::logout,
            commands::uninstall,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ======= END APP =======
