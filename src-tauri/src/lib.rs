mod commands;
mod email;
mod models;
mod storage;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AccountsState::default())
        .manage(SessionsState::default())
        .manage(EmailReceiverState::default())
        .invoke_handler(tauri::generate_handler![
            get_accounts,
            save_account,
            delete_account,
            update_last_login,
            save_browser_session,
            start_email_receiver,
            stop_email_receiver,
            get_verification_codes,
            get_email_receiver_status,
            test_email_connection,
            open_browser_window,
            close_browser_window,
            is_browser_window_open,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
