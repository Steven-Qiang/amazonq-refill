use crate::email::EmailReceiver;
use crate::models::{Account, BrowserSession, EmailReceiverStatus, VerificationCode};
use crate::storage::{load_json, save_json};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

pub type AccountsState = Mutex<Vec<Account>>;
pub type SessionsState = Mutex<Vec<BrowserSession>>;
pub type EmailReceiverState = Mutex<Option<EmailReceiver>>;

#[tauri::command]
pub async fn get_accounts(
    app: AppHandle,
    accounts_state: State<'_, AccountsState>,
) -> Result<Vec<Account>, String> {
    // 尝试从文件加载
    match load_json::<Vec<Account>>(&app, "accounts.json") {
        Ok(accounts) => {
            *accounts_state.lock().unwrap() = accounts.clone();
            Ok(accounts)
        }
        Err(_) => {
            // 文件不存在，返回空列表
            Ok(vec![])
        }
    }
}

#[tauri::command]
pub async fn save_account(
    app: AppHandle,
    accounts_state: State<'_, AccountsState>,
    account: Account,
) -> Result<(), String> {
    let mut accounts = accounts_state.lock().unwrap();

    // 查找是否已存在
    if let Some(pos) = accounts.iter().position(|a| a.id == account.id) {
        accounts[pos] = account;
    } else {
        accounts.push(account);
    }

    // 保存到文件
    save_json(&app, "accounts.json", &*accounts).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_account(
    app: AppHandle,
    accounts_state: State<'_, AccountsState>,
    id: String,
) -> Result<(), String> {
    let mut accounts = accounts_state.lock().unwrap();
    accounts.retain(|a| a.id != id);

    save_json(&app, "accounts.json", &*accounts).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn update_last_login(
    app: AppHandle,
    accounts_state: State<'_, AccountsState>,
    id: String,
) -> Result<(), String> {
    let mut accounts = accounts_state.lock().unwrap();

    if let Some(account) = accounts.iter_mut().find(|a| a.id == id) {
        account.last_login_time = Some(chrono::Utc::now().to_rfc3339());
    }

    save_json(&app, "accounts.json", &*accounts).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn save_browser_session(
    app: AppHandle,
    sessions_state: State<'_, SessionsState>,
    account_id: String,
) -> Result<(), String> {
    let mut sessions = sessions_state.lock().unwrap();

    let session = BrowserSession {
        account_id: account_id.clone(),
        cookies: None,
        local_storage: None,
    };

    // 移除旧的会话
    sessions.retain(|s| s.account_id != account_id);
    sessions.push(session);

    save_json(&app, "sessions.json", &*sessions).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn start_email_receiver(
    email_receiver_state: State<'_, EmailReceiverState>,
    email: String,
    password: String,
    server: String,
    port: u16,
) -> Result<(), String> {
    let mut receiver_guard = email_receiver_state.lock().unwrap();

    let receiver = EmailReceiver::new();
    receiver
        .start_receiving(email, password, server, port)
        .map_err(|e| e.to_string())?;

    *receiver_guard = Some(receiver);
    Ok(())
}

#[tauri::command]
pub async fn stop_email_receiver(
    email_receiver_state: State<'_, EmailReceiverState>,
) -> Result<(), String> {
    let mut receiver_guard = email_receiver_state.lock().unwrap();

    if let Some(receiver) = receiver_guard.as_ref() {
        receiver.stop_receiving();
    }

    *receiver_guard = None;
    Ok(())
}

#[tauri::command]
pub async fn get_verification_codes(
    email_receiver_state: State<'_, EmailReceiverState>,
) -> Result<Vec<VerificationCode>, String> {
    let receiver_guard = email_receiver_state.lock().unwrap();

    if let Some(receiver) = receiver_guard.as_ref() {
        Ok(receiver.get_codes())
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn get_email_receiver_status(
    email_receiver_state: State<'_, EmailReceiverState>,
) -> Result<EmailReceiverStatus, String> {
    let receiver_guard = email_receiver_state.lock().unwrap();

    if let Some(receiver) = receiver_guard.as_ref() {
        Ok(receiver.get_status())
    } else {
        Ok(EmailReceiverStatus {
            status: crate::models::EmailStatus::Idle,
            error_message: None,
            last_check_time: None,
            codes_count: 0,
        })
    }
}

#[tauri::command]
pub async fn test_email_connection(
    email: String,
    password: String,
    server: String,
    port: u16,
) -> Result<String, String> {
    use rust_pop3_client::Pop3Connection;

    if port == 110 {
        return Err(
            "Port 110 (plain POP3) is not supported. Please use port 995 (POP3 over SSL/TLS)"
                .to_string(),
        );
    }

    let mut connection = Pop3Connection::new(&server, port)
        .map_err(|e| format!("Failed to connect to {}:{} - {}", server, port, e))?;

    connection
        .login(&email, &password)
        .map_err(|e| format!("Login failed for {} - {}", email, e))?;

    Ok("Connection successful".to_string())
}

#[tauri::command]
pub async fn open_browser_window(
    app: AppHandle,
    url: String,
    account_id: String,
) -> Result<(), String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};

    let window_label = format!("browser_{}", account_id);

    // 先关闭已存在的窗口
    let windows = app.webview_windows();
    if let Some(existing_window) = windows.get(&window_label) {
        let _ = existing_window.close();
        // 等待窗口关闭
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // 为每个账号创建独立的数据目录
    let app_data_dir = crate::storage::get_app_data_dir(&app).map_err(|e| e.to_string())?;
    let account_data_dir = app_data_dir.join("browser_data").join(&account_id);

    // 确保目录存在
    std::fs::create_dir_all(&account_data_dir).map_err(|e| e.to_string())?;

    WebviewWindowBuilder::new(
        &app,
        &window_label,
        WebviewUrl::External(url.parse().unwrap()),
    )
    .title("登录浏览器")
    .inner_size(1200.0, 800.0)
    .data_directory(account_data_dir)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn close_browser_window(app: AppHandle) -> Result<(), String> {
    // 获取所有窗口并关闭 browser_ 开头的窗口
    let windows = app.webview_windows();
    for (label, window) in windows {
        if label.starts_with("browser_") {
            window.close().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn is_browser_window_open(app: AppHandle, account_id: String) -> Result<bool, String> {
    let window_label = format!("browser_{}", account_id);
    let windows: std::collections::HashMap<String, tauri::WebviewWindow> = app.webview_windows();
    Ok(windows.contains_key(&window_label))
}
