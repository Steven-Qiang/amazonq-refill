use crate::models::{EmailReceiverStatus, EmailStatus, VerificationCode};
use mail_parser::MessageParser;
use regex::Regex;
use rust_pop3_client::Pop3Connection;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct EmailReceiver {
    pub codes: Arc<Mutex<VecDeque<VerificationCode>>>,
    pub is_running: Arc<Mutex<bool>>,
    pub status: Arc<Mutex<EmailReceiverStatus>>,
}

impl EmailReceiver {
    pub fn new() -> Self {
        Self {
            codes: Arc::new(Mutex::new(VecDeque::new())),
            is_running: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new(EmailReceiverStatus {
                status: EmailStatus::Idle,
                error_message: None,
                last_check_time: None,
                codes_count: 0,
            })),
        }
    }

    pub fn get_status(&self) -> EmailReceiverStatus {
        self.status.lock().unwrap().clone()
    }

    fn update_status(&self, status: EmailStatus, error_message: Option<String>) {
        let mut status_guard = self.status.lock().unwrap();
        status_guard.status = status;
        status_guard.error_message = error_message;
        status_guard.last_check_time = Some(chrono::Utc::now().timestamp_millis());
        status_guard.codes_count = self.codes.lock().unwrap().len();
    }

    pub fn start_receiving(
        &self,
        email: String,
        password: String,
        server: String,
        port: u16,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let codes = Arc::clone(&self.codes);
        let is_running = Arc::clone(&self.is_running);
        let status = Arc::clone(&self.status);

        *is_running.lock().unwrap() = true;

        Self::test_connection(&email, &password, &server, port)?;
        self.update_status(EmailStatus::Connecting, None);

        tokio::spawn(async move {
            if let Err(e) = Self::email_loop(
                email,
                password,
                server,
                port,
                codes,
                is_running,
                status.clone(),
            )
            .await
            {
                eprintln!("Email receiver error: {}", e);
                let mut status_guard = status.lock().unwrap();
                status_guard.status = EmailStatus::Error;
                status_guard.error_message = Some(e.to_string());
            }
        });

        Ok(())
    }

    pub fn stop_receiving(&self) {
        *self.is_running.lock().unwrap() = false;
        self.update_status(EmailStatus::Stopped, None);
    }

    pub fn get_codes(&self) -> Vec<VerificationCode> {
        let codes = self.codes.lock().unwrap();
        // 更新状态中的codes_count
        let mut status_guard = self.status.lock().unwrap();
        status_guard.codes_count = codes.len();
        codes.iter().cloned().collect()
    }

    fn test_connection(
        email: &str,
        password: &str,
        server: &str,
        port: u16,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if port == 110 {
            return Err(
                "Port 110 (plain POP3) is not supported. Please use port 995 (POP3 over SSL/TLS)"
                    .into(),
            );
        }

        let mut connection = Pop3Connection::new(server, port)
            .map_err(|e| format!("Failed to connect to {}:{} - {}", server, port, e))?;

        connection
            .login(email, password)
            .map_err(|e| format!("Login failed for {} - {}", email, e))?;

        Ok(())
    }

    async fn email_loop(
        email: String,
        password: String,
        server: String,
        port: u16,
        codes: Arc<Mutex<VecDeque<VerificationCode>>>,
        is_running: Arc<Mutex<bool>>,
        status: Arc<Mutex<EmailReceiverStatus>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        {
            let mut status_guard = status.lock().unwrap();
            status_guard.status = EmailStatus::Connected;
            status_guard.error_message = None;
        }

        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: u32 = 3;

        while *is_running.lock().unwrap() {
            // 更新为接收中
            {
                let mut status_guard = status.lock().unwrap();
                status_guard.status = EmailStatus::Receiving;
            }

            match Self::check_emails(&email, &password, &server, port, &codes) {
                Ok(_) => {
                    consecutive_errors = 0;
                    let mut status_guard = status.lock().unwrap();
                    status_guard.error_message = None;
                    status_guard.last_check_time = Some(chrono::Utc::now().timestamp_millis());
                    status_guard.codes_count = codes.lock().unwrap().len();
                }
                Err(e) => {
                    consecutive_errors += 1;
                    eprintln!("Email check error ({}): {}", consecutive_errors, e);

                    let mut status_guard = status.lock().unwrap();
                    status_guard.error_message = Some(e.clone());

                    if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                        status_guard.status = EmailStatus::Error;
                        return Err(format!("Too many consecutive errors: {}", e).into());
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }

        Ok(())
    }

    fn check_emails(
        email: &str,
        password: &str,
        server: &str,
        port: u16,
        codes: &Arc<Mutex<VecDeque<VerificationCode>>>,
    ) -> Result<(), String> {
        if port == 110 {
            return Err(
                "Port 110 (plain POP3) is not supported. Please use port 995 (POP3 over SSL/TLS)"
                    .to_string(),
            );
        }

        let mut connection = Pop3Connection::new(server, port)
            .map_err(|e| format!("Connection failed to {}:{} - {}", server, port, e))?;

        connection
            .login(email, password)
            .map_err(|e| format!("Login failed for {} - {}", email, e))?;
        let infos = connection
            .list()
            .map_err(|e| format!("Failed to list emails - {}", e))?;

        for info in infos.iter() {
            let mut buffer = Vec::new();
            if connection.retrieve(info.message_id, &mut buffer).is_ok() {
                // 使用mail-parser解析邮件
                let parser = MessageParser::default();
                if let Some(message) = parser.parse(&buffer) {
                    let from = message
                        .from()
                        .and_then(|f| f.first())
                        .and_then(|addr| addr.address())
                        .unwrap_or("");

                    // 只处理来自 no-reply@login.awsapps.com 的邮件
                    if !from.contains("no-reply@login.awsapps.com") {
                        continue;
                    }

                    let subject = message.subject().unwrap_or("");

                    // 获取邮件正文
                    let body = message
                        .body_text(0)
                        .map(|s| s.to_string())
                        .unwrap_or_default();

                    // 如果没有纯文本，尝试获取HTML
                    let html_body = if body.is_empty() {
                        message
                            .body_html(0)
                            .map(|s| s.to_string())
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };

                    let search_text = if !body.is_empty() { &body } else { &html_body };
                    if let Some(code) = Self::extract_verification_code(search_text) {
                        println!("✓ Found verification code: {}", code);

                        let email_time = message
                            .date()
                            .and_then(|d| chrono::DateTime::parse_from_rfc2822(&d.to_rfc822()).ok())
                            .map(|dt| dt.timestamp_millis())
                            .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

                        let verification_code = VerificationCode {
                            code,
                            timestamp: email_time,
                            from: from.to_string(),
                            subject: subject.to_string(),
                        };

                        let mut codes_guard = codes.lock().unwrap();

                        if !codes_guard.iter().any(|c| c.code == verification_code.code) {
                            codes_guard.push_front(verification_code);
                            if codes_guard.len() > 10 {
                                codes_guard.pop_back();
                            }
                        }
                    } else {
                        println!("✗ No verification code found");
                    }
                }
            }
        }

        Ok(())
    }

    fn extract_verification_code(text: &str) -> Option<String> {
        // 从 HTML 中提取 <div class="code">148885</div>
        if let Ok(re) = Regex::new(r#"<div class=3D"code"[^>]*>(\d{4,8})</div>"#) {
            if let Some(captures) = re.captures(text) {
                if let Some(code) = captures.get(1) {
                    return Some(code.as_str().to_string());
                }
            }
        }

        // 简单粗暴：找所有6位数字
        if let Ok(re) = Regex::new(r"\b(\d{6})\b") {
            for captures in re.captures_iter(text) {
                if let Some(code) = captures.get(1) {
                    let code_str = code.as_str();
                    // 排除全0和以20开头的（年份）
                    if code_str.chars().any(|c| c != '0') && !code_str.starts_with("20") {
                        return Some(code_str.to_string());
                    }
                }
            }
        }
        None
    }
}
