use super::{AUTH_ENDPOINT, KEYRING_ACCOUNT, KEYRING_SERVICE, SCOPE, TOKEN_ENDPOINT};
use crate::settings;
use crate::AppState;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    /// Unix timestamp (seconds) when the access token expires.
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GoogleStatus {
    /// A GOOGLE_CLIENT_ID is present in the environment.
    pub configured: bool,
    /// Tokens are stored, i.e. the user has connected.
    pub connected: bool,
}

pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: Option<String>,
}

impl OAuthConfig {
    pub fn from_env() -> Result<Self, String> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| {
                "GOOGLE_CLIENT_ID 환경변수가 설정되지 않았습니다. README의 연동 설정을 참고하세요."
                    .to_string()
            })?;
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .ok()
            .filter(|s| !s.trim().is_empty());
        Ok(Self {
            client_id,
            client_secret,
        })
    }
}

// ---- keyring token storage -------------------------------------------------

fn entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT).map_err(|e| e.to_string())
}

pub fn store_tokens(tokens: &Tokens) -> Result<(), String> {
    let json = serde_json::to_string(tokens).map_err(|e| e.to_string())?;
    entry()?.set_password(&json).map_err(|e| e.to_string())
}

pub fn load_tokens() -> Result<Option<Tokens>, String> {
    match entry()?.get_password() {
        Ok(json) => {
            let tokens: Tokens = serde_json::from_str(&json).map_err(|e| e.to_string())?;
            Ok(Some(tokens))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn delete_tokens() -> Result<(), String> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Return a non-expired access token, refreshing it first if needed.
pub fn valid_access_token() -> Result<String, String> {
    let tokens = load_tokens()?.ok_or_else(|| "Google 계정이 연결되어 있지 않습니다.".to_string())?;
    if tokens.expires_at - Utc::now().timestamp() > 60 {
        return Ok(tokens.access_token);
    }
    let refreshed = refresh(&tokens)?;
    store_tokens(&refreshed)?;
    Ok(refreshed.access_token)
}

// ---- PKCE helpers ----------------------------------------------------------

fn random_token(len: usize) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

fn code_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<i64>,
}

/// Exchange a refresh token for a fresh access token (preserving the refresh token).
fn refresh(tokens: &Tokens) -> Result<Tokens, String> {
    let refresh_token = tokens
        .refresh_token
        .clone()
        .ok_or_else(|| "refresh token이 없습니다. Google 계정을 다시 연결하세요.".to_string())?;
    let cfg = OAuthConfig::from_env()?;

    let mut form = vec![
        ("grant_type", "refresh_token".to_string()),
        ("refresh_token", refresh_token.clone()),
        ("client_id", cfg.client_id),
    ];
    if let Some(secret) = cfg.client_secret {
        form.push(("client_secret", secret));
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(TOKEN_ENDPOINT)
        .form(&form)
        .send()
        .map_err(|e| format!("network: 토큰 갱신 요청 실패: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("auth: 토큰 갱신 실패 ({status}): {body}"));
    }
    let parsed: TokenResponse = resp.json().map_err(|e| e.to_string())?;
    Ok(Tokens {
        access_token: parsed.access_token,
        // Google omits refresh_token on refresh; keep the existing one.
        refresh_token: parsed.refresh_token.or(Some(refresh_token)),
        expires_at: Utc::now().timestamp() + parsed.expires_in.unwrap_or(3600),
    })
}

/// Run the full Authorization Code + PKCE flow on a blocking thread.
fn run_oauth_flow(app: &AppHandle, cfg: &OAuthConfig) -> Result<Tokens, String> {
    let verifier = random_token(64);
    let challenge = code_challenge(&verifier);
    let csrf_state = random_token(24);

    // Loopback server on a random free port.
    let server = tiny_http::Server::http("127.0.0.1:0")
        .map_err(|e| format!("loopback 서버 시작 실패: {e}"))?;
    let port = server
        .server_addr()
        .to_ip()
        .ok_or_else(|| "loopback 주소를 확인할 수 없습니다.".to_string())?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}");

    // Build the consent URL.
    let mut auth_url = Url::parse(AUTH_ENDPOINT).map_err(|e| e.to_string())?;
    auth_url
        .query_pairs_mut()
        .append_pair("client_id", &cfg.client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", SCOPE)
        .append_pair("code_challenge", &challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent")
        .append_pair("state", &csrf_state);

    app.opener()
        .open_url(auth_url.to_string(), None::<String>)
        .map_err(|e| format!("브라우저를 열 수 없습니다: {e}"))?;

    // Wait (up to 5 min) for Google to redirect back with the code.
    let deadline = std::time::Instant::now() + Duration::from_secs(300);
    let code = loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            return Err("로그인 시간이 초과되었습니다. 다시 시도하세요.".to_string());
        }
        let request = match server
            .recv_timeout(remaining)
            .map_err(|e| e.to_string())?
        {
            Some(r) => r,
            None => return Err("로그인 시간이 초과되었습니다. 다시 시도하세요.".to_string()),
        };

        let full = format!("http://127.0.0.1{}", request.url());
        let parsed = Url::parse(&full).map_err(|e| e.to_string())?;
        let mut got_code = None;
        let mut got_state = None;
        let mut got_error = None;
        for (k, v) in parsed.query_pairs() {
            match k.as_ref() {
                "code" => got_code = Some(v.to_string()),
                "state" => got_state = Some(v.to_string()),
                "error" => got_error = Some(v.to_string()),
                _ => {}
            }
        }

        let body = "<html><head><meta charset=\"utf-8\"></head><body style=\"font-family:sans-serif;text-align:center;padding-top:3rem\"><h2>duckCalendar 연결 처리 완료</h2><p>이 창을 닫고 앱으로 돌아가세요.</p></body></html>";
        let response = tiny_http::Response::from_string(body).with_header(
            tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
                .unwrap(),
        );
        let _ = request.respond(response);

        if let Some(err) = got_error {
            return Err(format!("permission: 권한 동의가 거부되었습니다 ({err})."));
        }
        if let Some(code) = got_code {
            if got_state.as_deref() != Some(csrf_state.as_str()) {
                return Err("auth: state 불일치 (CSRF 의심). 다시 시도하세요.".to_string());
            }
            break code;
        }
        // Ignore unrelated requests (e.g. favicon) and keep waiting.
    };

    // Exchange the authorization code for tokens.
    let mut form = vec![
        ("grant_type", "authorization_code".to_string()),
        ("code", code),
        ("client_id", cfg.client_id.clone()),
        ("redirect_uri", redirect_uri),
        ("code_verifier", verifier),
    ];
    if let Some(secret) = &cfg.client_secret {
        form.push(("client_secret", secret.clone()));
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(TOKEN_ENDPOINT)
        .form(&form)
        .send()
        .map_err(|e| format!("network: 토큰 교환 요청 실패: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("auth: 토큰 교환 실패 ({status}): {body}"));
    }
    let parsed: TokenResponse = resp.json().map_err(|e| e.to_string())?;
    Ok(Tokens {
        access_token: parsed.access_token,
        refresh_token: parsed.refresh_token,
        expires_at: Utc::now().timestamp() + parsed.expires_in.unwrap_or(3600),
    })
}

// ---- Tauri commands --------------------------------------------------------

#[tauri::command]
pub async fn google_connect(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let cfg = OAuthConfig::from_env()?;
    let app_for_flow = app.clone();
    let tokens = tauri::async_runtime::spawn_blocking(move || run_oauth_flow(&app_for_flow, &cfg))
        .await
        .map_err(|e| e.to_string())??;
    store_tokens(&tokens)?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    settings::set(&conn, "google_connected", "true").map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn google_disconnect(state: State<'_, AppState>) -> Result<(), String> {
    delete_tokens()?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    settings::set(&conn, "google_connected", "false").map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn google_status() -> Result<GoogleStatus, String> {
    tauri::async_runtime::spawn_blocking(|| {
        Ok(GoogleStatus {
            configured: OAuthConfig::from_env().is_ok(),
            connected: load_tokens()?.is_some(),
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
