//! Explicit screenshot sharing to a local or trusted-LAN LanChat room.
//!
//! This module owns all HTTP access. Renderers receive only the resulting room
//! name and message metadata, never local library paths or network clients.

use std::{
    fs,
    io::Write,
    net::IpAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use chrono::TimeZone;
use reqwest::{
    header::{ACCEPT, CONTENT_TYPE, COOKIE, SET_COOKIE},
    redirect::Policy,
    Client, Response, StatusCode,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use url::Url;

use crate::{
    core::asset::{CaptureAsset, CaptureKind},
    state::AppState,
};

const PRODUCT: &str = "lan-chat";
const PROTOCOL: u32 = 1;
const CONFIG_FILENAME: &str = "lanchat-share.json";
const DISCOVERY_PORTS: [u16; 2] = [81, 8787];
const PROBE_TIMEOUT: Duration = Duration::from_millis(650);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const UPLOAD_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_HEALTH_BYTES: u64 = 64 * 1024;
const MAX_RESPONSE_BYTES: u64 = 64 * 1024;
const MAX_SHARE_BYTES: u64 = 500_000_000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanChatShareSettingsDto {
    pub origin: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanChatConnectionDto {
    pub origin: String,
    pub room_name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanChatShareResult {
    pub origin: String,
    pub room_name: String,
    pub message_id: i64,
    pub file_name: String,
    pub file_size: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct SharePreferences {
    origin: Option<String>,
    identity: Option<IdentityPreference>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct IdentityPreference {
    room_id: String,
    cookie: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VerifiedRoom {
    origin: String,
    room_id: String,
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    product: String,
    protocol: u32,
    room_id: String,
    instance_id: String,
    name: String,
    port: u16,
    #[serde(default)]
    version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageResponse {
    id: i64,
    #[serde(default)]
    file_name: String,
    #[serde(default)]
    file_size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeFailure {
    Unreachable,
    WrongService,
    IncompatibleProtocol,
    InvalidRoom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LanChatShareError {
    NotRunning,
    InvalidAddress,
    IncompatibleProtocol,
    UploadFailed,
    InvalidAsset,
}

impl LanChatShareError {
    fn message(self) -> &'static str {
        match self {
            Self::NotRunning => "LanChat is not running. Start it and try again.",
            Self::InvalidAddress => "The LanChat address is invalid.",
            Self::IncompatibleProtocol => "This LanChat room uses an incompatible protocol.",
            Self::UploadFailed => "Could not send this screenshot to LanChat.",
            Self::InvalidAsset => "This image could not be sent to LanChat.",
        }
    }
}

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join(CONFIG_FILENAME))
        .map_err(|error| error.to_string())
}

fn load_preferences(path: &Path) -> SharePreferences {
    let mut preferences = fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<SharePreferences>(&bytes).ok())
        .unwrap_or_default();
    preferences.origin = preferences
        .origin
        .as_deref()
        .and_then(|origin| normalize_origin(origin).ok().flatten());
    preferences.identity = preferences.identity.filter(|identity| {
        uuid::Uuid::parse_str(&identity.room_id).is_ok() && cookie_is_valid(&identity.cookie)
    });
    preferences
}

fn save_preferences(path: &Path, preferences: &SharePreferences) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "The LanChat settings path is invalid.".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let data = serde_json::to_vec_pretty(preferences).map_err(|error| error.to_string())?;
    let mut temporary = tempfile::Builder::new()
        .prefix(".lanchat-share-")
        .suffix(".tmp")
        .tempfile_in(parent)
        .map_err(|error| error.to_string())?;
    temporary
        .write_all(&data)
        .map_err(|error| error.to_string())?;
    temporary
        .as_file_mut()
        .sync_all()
        .map_err(|error| error.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        temporary
            .as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|error| error.to_string())?;
    }
    temporary
        .persist(path)
        .map_err(|error| error.error.to_string())?;
    #[cfg(unix)]
    if let Ok(directory) = fs::File::open(parent) {
        let _ = directory.sync_all();
    }
    Ok(())
}

fn normalize_origin(value: &str) -> Result<Option<String>, LanChatShareError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    let candidate = if value.starts_with("http://") || value.starts_with("https://") {
        value.to_string()
    } else {
        format!("http://{value}")
    };
    let url = Url::parse(&candidate).map_err(|_| LanChatShareError::InvalidAddress)?;
    if url.scheme() != "http"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !matches!(url.path(), "" | "/")
    {
        return Err(LanChatShareError::InvalidAddress);
    }
    let host = url.host_str().ok_or(LanChatShareError::InvalidAddress)?;
    let host = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(host);
    if !is_trusted_lan_host(host) {
        return Err(LanChatShareError::InvalidAddress);
    }
    let normalized = url.as_str().trim_end_matches('/').to_string();
    Ok(Some(normalized))
}

fn is_trusted_lan_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    match host.parse::<IpAddr>() {
        Ok(IpAddr::V4(address)) => {
            address.is_loopback() || address.is_private() || address.is_link_local()
        }
        Ok(IpAddr::V6(address)) => {
            address.is_loopback() || address.is_unique_local() || address.is_unicast_link_local()
        }
        Err(_) => false,
    }
}

fn probe_client() -> Result<Client, LanChatShareError> {
    ensure_crypto_provider();
    Client::builder()
        .redirect(Policy::none())
        .retry(reqwest::retry::never())
        .no_proxy()
        .connect_timeout(PROBE_TIMEOUT)
        .timeout(PROBE_TIMEOUT)
        .build()
        .map_err(|_| LanChatShareError::NotRunning)
}

fn upload_client() -> Result<Client, LanChatShareError> {
    ensure_crypto_provider();
    Client::builder()
        .redirect(Policy::none())
        .retry(reqwest::retry::never())
        .no_proxy()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(UPLOAD_TIMEOUT)
        .build()
        .map_err(|_| LanChatShareError::UploadFailed)
}

fn ensure_crypto_provider() {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
}

async fn probe_room(client: &Client, origin: &str) -> Result<VerifiedRoom, ProbeFailure> {
    let url =
        Url::parse(&format!("{origin}/api/health")).map_err(|_| ProbeFailure::WrongService)?;
    let response = client
        .get(url.clone())
        .header(ACCEPT, "application/json")
        .send()
        .await
        .map_err(|_| ProbeFailure::Unreachable)?;
    if !response.status().is_success() {
        return Err(ProbeFailure::WrongService);
    }
    let bytes = read_limited_response(response, MAX_HEALTH_BYTES)
        .await
        .map_err(|_| ProbeFailure::InvalidRoom)?;
    let health =
        serde_json::from_slice::<HealthResponse>(&bytes).map_err(|_| ProbeFailure::InvalidRoom)?;
    let expected_port = url
        .port_or_known_default()
        .ok_or(ProbeFailure::InvalidRoom)?;
    validate_health(&health, expected_port)?;
    let name = health.name.trim();
    Ok(VerifiedRoom {
        origin: origin.to_string(),
        room_id: health.room_id,
        name: if name.is_empty() {
            "LanChat".to_string()
        } else {
            name.chars().take(80).collect()
        },
        version: health.version.trim().chars().take(40).collect(),
    })
}

fn validate_health(health: &HealthResponse, expected_port: u16) -> Result<(), ProbeFailure> {
    if health.product != PRODUCT {
        return Err(ProbeFailure::WrongService);
    }
    if health.protocol != PROTOCOL {
        return Err(ProbeFailure::IncompatibleProtocol);
    }
    if health.port != expected_port
        || uuid::Uuid::parse_str(&health.room_id).is_err()
        || uuid::Uuid::parse_str(&health.instance_id).is_err()
    {
        return Err(ProbeFailure::InvalidRoom);
    }
    Ok(())
}

async fn discover_room(preferences: &SharePreferences) -> Result<VerifiedRoom, LanChatShareError> {
    let client = probe_client()?;
    let mut explicit_failure = None;
    if let Some(origin) = preferences.origin.as_deref() {
        match probe_room(&client, origin).await {
            Ok(room) => return Ok(room),
            Err(failure) => explicit_failure = Some(failure),
        }
    }

    let probe = |port: u16| {
        let client = client.clone();
        async move {
            let origin = format!("http://127.0.0.1:{port}");
            probe_room(&client, &origin).await.ok()
        }
    };
    let (port_81, port_8787) = tokio::join!(probe(DISCOVERY_PORTS[0]), probe(DISCOVERY_PORTS[1]));
    if let Some(room) = port_81.or(port_8787) {
        return Ok(room);
    }

    match explicit_failure {
        Some(ProbeFailure::WrongService | ProbeFailure::InvalidRoom) => {
            Err(LanChatShareError::InvalidAddress)
        }
        Some(ProbeFailure::IncompatibleProtocol) => Err(LanChatShareError::IncompatibleProtocol),
        Some(ProbeFailure::Unreachable) | None => Err(LanChatShareError::NotRunning),
    }
}

async fn read_limited_response(mut response: Response, limit: u64) -> Result<Vec<u8>, ()> {
    if response
        .content_length()
        .is_some_and(|length| length > limit)
    {
        return Err(());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| ())? {
        if bytes.len().saturating_add(chunk.len()) as u64 > limit {
            return Err(());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn encode_file_name(file_name: &str) -> String {
    percent_encoding::utf8_percent_encode(file_name, percent_encoding::NON_ALPHANUMERIC).to_string()
}

fn cookie_is_valid(cookie: &str) -> bool {
    let Some(value) = cookie.strip_prefix("lan_identity=") else {
        return false;
    };
    uuid::Uuid::parse_str(value).is_ok()
}

fn identity_cookie(response: &Response) -> Option<String> {
    response
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .find_map(|value| {
            let value = value.to_str().ok()?;
            let first = value.split(';').next()?.trim();
            let (name, cookie_value) = first.split_once('=')?;
            if name.eq_ignore_ascii_case("lan_identity")
                && uuid::Uuid::parse_str(cookie_value.trim()).is_ok()
            {
                Some(format!("lan_identity={}", cookie_value.trim()))
            } else {
                None
            }
        })
}

fn remember_identity(preferences: &mut SharePreferences, room: &VerifiedRoom, cookie: String) {
    preferences.identity = Some(IdentityPreference {
        room_id: room.room_id.clone(),
        cookie,
    });
}

async fn share_image(
    config_path: &Path,
    display_name: &str,
    bytes: &[u8],
) -> Result<LanChatShareResult, LanChatShareError> {
    if display_name.trim().is_empty() || bytes.is_empty() {
        return Err(LanChatShareError::InvalidAsset);
    }
    if bytes.len() as u64 > MAX_SHARE_BYTES {
        return Err(LanChatShareError::InvalidAsset);
    }

    let mut preferences = load_preferences(config_path);
    let room = discover_room(&preferences).await?;
    let existing_identity = preferences
        .identity
        .as_ref()
        .filter(|identity| identity.room_id == room.room_id)
        .map(|identity| identity.cookie.clone());
    let client = upload_client()?;
    let url = Url::parse(&format!("{}/api/files", room.origin))
        .map_err(|_| LanChatShareError::InvalidAddress)?;
    let mut request = client
        .post(url)
        .header(CONTENT_TYPE, "application/octet-stream")
        .header("X-File-Name", encode_file_name(display_name))
        .body(bytes.to_vec());
    if let Some(cookie) = existing_identity {
        request = request.header(COOKIE, cookie);
    }
    let response = request
        .send()
        .await
        .map_err(|_| LanChatShareError::UploadFailed)?;
    let status = response.status();
    let updated_cookie = identity_cookie(&response);
    let response_bytes = read_limited_response(response, MAX_RESPONSE_BYTES)
        .await
        .map_err(|_| LanChatShareError::UploadFailed)?;

    if status != StatusCode::CREATED {
        return Err(LanChatShareError::UploadFailed);
    }
    let message = serde_json::from_slice::<MessageResponse>(&response_bytes)
        .map_err(|_| LanChatShareError::UploadFailed)?;
    if let Some(cookie) = updated_cookie {
        remember_identity(&mut preferences, &room, cookie);
        // The room has already accepted the message. A preference-write failure
        // must not invite the user to retry and create a duplicate upload.
        let _ = save_preferences(config_path, &preferences);
    }
    Ok(LanChatShareResult {
        origin: room.origin,
        room_name: room.name,
        message_id: message.id,
        file_name: if message.file_name.trim().is_empty() {
            display_name.to_string()
        } else {
            message.file_name
        },
        file_size: if message.file_size == 0 {
            bytes.len() as u64
        } else {
            message.file_size
        },
    })
}

fn share_display_name(asset: &CaptureAsset) -> String {
    let stamp = chrono::Local
        .timestamp_millis_opt(asset.created_at as i64)
        .single()
        .map(|time| time.format("%Y%m%d-%H%M%S").to_string())
        .unwrap_or_else(|| "capture".to_string());
    let extension = Path::new(&asset.filename)
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| {
            !value.is_empty()
                && value.len() <= 10
                && value
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
        })
        .unwrap_or("png");
    format!(
        "kaka-{stamp}-{}.{}",
        &asset.id.simple().to_string()[..8],
        extension
    )
}

fn asset_share_path(app: &AppHandle, id: &str) -> Result<(PathBuf, String), String> {
    let parsed =
        uuid::Uuid::parse_str(id).map_err(|_| "The capture could not be found.".to_string())?;
    let state = app.state::<AppState>();
    let mut context = state.library.lock().unwrap();
    let library = context.library().map_err(|error| error.to_string())?;
    let asset = library
        .asset_by_id(&parsed)
        .cloned()
        .ok_or_else(|| "The capture could not be found.".to_string())?;
    if asset.kind != CaptureKind::Image || asset.trashed_at.is_some() {
        return Err(LanChatShareError::InvalidAsset.message().to_string());
    }
    let path = library
        .readable_asset_url(&asset)
        .map_err(|_| LanChatShareError::InvalidAsset.message().to_string())?;
    Ok((path, share_display_name(&asset)))
}

#[tauri::command]
pub fn get_lanchat_share_settings(app: AppHandle) -> Result<LanChatShareSettingsDto, String> {
    let preferences = load_preferences(&config_path(&app)?);
    Ok(LanChatShareSettingsDto {
        origin: preferences.origin,
    })
}

#[tauri::command]
pub fn set_lanchat_share_origin(
    app: AppHandle,
    origin: Option<String>,
) -> Result<LanChatShareSettingsDto, String> {
    let normalized = match origin.as_deref() {
        Some(value) => normalize_origin(value).map_err(|error| error.message().to_string())?,
        None => None,
    };
    let path = config_path(&app)?;
    let mut preferences = load_preferences(&path);
    preferences.origin = normalized.clone();
    save_preferences(&path, &preferences)?;
    Ok(LanChatShareSettingsDto { origin: normalized })
}

#[tauri::command]
pub async fn check_lanchat_share(app: AppHandle) -> Result<LanChatConnectionDto, String> {
    let preferences = load_preferences(&config_path(&app)?);
    let room = discover_room(&preferences)
        .await
        .map_err(|error| error.message().to_string())?;
    Ok(LanChatConnectionDto {
        origin: room.origin,
        room_name: room.name,
        version: room.version,
    })
}

#[tauri::command]
pub async fn share_asset_to_lanchat(
    app: AppHandle,
    id: String,
) -> Result<LanChatShareResult, String> {
    share_saved_asset(&app, &id).await
}

pub(crate) async fn share_saved_asset(
    app: &AppHandle,
    id: &str,
) -> Result<LanChatShareResult, String> {
    let (path, display_name) = asset_share_path(app, id)?;
    let metadata =
        fs::metadata(&path).map_err(|_| LanChatShareError::InvalidAsset.message().to_string())?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_SHARE_BYTES {
        return Err(LanChatShareError::InvalidAsset.message().to_string());
    }
    let bytes = tauri::async_runtime::spawn_blocking(move || fs::read(path))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|_| LanChatShareError::InvalidAsset.message().to_string())?;
    share_image(&config_path(app)?, &display_name, &bytes)
        .await
        .map_err(|error| error.message().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::{Arc, Mutex},
        thread,
    };

    #[derive(Debug)]
    struct CapturedRequest {
        target: String,
        headers: String,
        body: Vec<u8>,
    }

    fn read_request(stream: &mut std::net::TcpStream) -> CapturedRequest {
        let mut data = Vec::new();
        let mut buffer = [0_u8; 4096];
        let header_end = loop {
            let read = stream.read(&mut buffer).unwrap();
            assert!(read > 0);
            data.extend_from_slice(&buffer[..read]);
            if let Some(index) = data.windows(4).position(|window| window == b"\r\n\r\n") {
                break index + 4;
            }
        };
        let headers = String::from_utf8_lossy(&data[..header_end]).to_string();
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().ok())
                    .flatten()
            })
            .unwrap_or(0);
        while data.len() < header_end + content_length {
            let read = stream.read(&mut buffer).unwrap();
            assert!(read > 0);
            data.extend_from_slice(&buffer[..read]);
        }
        let target = headers
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap()
            .to_string();
        CapturedRequest {
            target,
            headers,
            body: data[header_end..header_end + content_length].to_vec(),
        }
    }

    fn write_json(stream: &mut std::net::TcpStream, status: &str, body: &str, cookie: bool) {
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n{}",
            body.len(),
            if cookie {
                "Set-Cookie: lan_identity=11111111-1111-4111-8111-111111111111; HttpOnly; SameSite=Strict; Path=/\r\n"
            } else {
                ""
            },
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    }

    #[test]
    fn normalizes_safe_local_and_private_origins() {
        assert_eq!(
            normalize_origin("127.0.0.1:8787").unwrap().as_deref(),
            Some("http://127.0.0.1:8787")
        );
        assert_eq!(
            normalize_origin("http://192.168.1.8:81/")
                .unwrap()
                .as_deref(),
            Some("http://192.168.1.8:81")
        );
        assert_eq!(
            normalize_origin("[::1]:8787").unwrap().as_deref(),
            Some("http://[::1]:8787")
        );
    }

    #[test]
    fn rejects_public_https_and_credentials() {
        for value in [
            "https://127.0.0.1:8787",
            "http://example.com:8787",
            "http://user:pass@127.0.0.1:8787",
            "http://127.0.0.1:8787/api",
        ] {
            assert_eq!(
                normalize_origin(value),
                Err(LanChatShareError::InvalidAddress)
            );
        }
    }

    #[test]
    fn extracts_only_a_valid_lanchat_identity_cookie() {
        let cookie = "lan_identity=11111111-1111-4111-8111-111111111111";
        assert!(cookie_is_valid(cookie));
        assert!(!cookie_is_valid("lan_identity=not-a-uuid"));
        assert!(!cookie_is_valid(
            "other=11111111-1111-4111-8111-111111111111"
        ));
    }

    #[test]
    fn preference_writes_replace_the_file_without_leaving_temporary_content() {
        let directory = tempfile::tempdir().unwrap();
        let config = directory.path().join(CONFIG_FILENAME);
        save_preferences(
            &config,
            &SharePreferences {
                origin: Some("http://127.0.0.1:8787".to_string()),
                identity: None,
            },
        )
        .unwrap();
        save_preferences(
            &config,
            &SharePreferences {
                origin: Some("http://192.168.1.8:81".to_string()),
                identity: None,
            },
        )
        .unwrap();

        assert_eq!(
            load_preferences(&config).origin.as_deref(),
            Some("http://192.168.1.8:81")
        );
        let names = fs::read_dir(directory.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        assert_eq!(names, vec![std::ffi::OsString::from(CONFIG_FILENAME)]);
    }

    #[test]
    fn file_names_are_utf8_header_safe() {
        assert_eq!(
            encode_file_name("截图 2026.png"),
            "%E6%88%AA%E5%9B%BE%202026%2Epng"
        );
    }

    #[test]
    fn validates_product_protocol_port_and_room_ids() {
        let mut health = HealthResponse {
            product: PRODUCT.to_string(),
            protocol: PROTOCOL,
            room_id: "22222222-2222-4222-8222-222222222222".to_string(),
            instance_id: "33333333-3333-4333-8333-333333333333".to_string(),
            name: "同频 · test".to_string(),
            port: 8787,
            version: "1.3.2".to_string(),
        };
        assert_eq!(validate_health(&health, 8787), Ok(()));
        health.product = "another-app".to_string();
        assert_eq!(
            validate_health(&health, 8787),
            Err(ProbeFailure::WrongService)
        );
        health.product = PRODUCT.to_string();
        health.protocol = 2;
        assert_eq!(
            validate_health(&health, 8787),
            Err(ProbeFailure::IncompatibleProtocol)
        );
        health.protocol = PROTOCOL;
        assert_eq!(validate_health(&health, 81), Err(ProbeFailure::InvalidRoom));
    }

    #[tokio::test]
    async fn shares_twice_and_reuses_the_room_identity() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let captured = Arc::new(Mutex::new(Vec::<CapturedRequest>::new()));
        let server_captured = Arc::clone(&captured);
        let server = thread::spawn(move || {
            let mut upload_number = 0;
            for _ in 0..4 {
                let (mut stream, _) = listener.accept().unwrap();
                let request = read_request(&mut stream);
                if request.target == "/api/health" {
                    let body = format!(
                        r#"{{"product":"lan-chat","protocol":1,"roomId":"22222222-2222-4222-8222-222222222222","instanceId":"33333333-3333-4333-8333-333333333333","name":"同频 · test","port":{port},"version":"1.3.2"}}"#
                    );
                    write_json(&mut stream, "200 OK", &body, false);
                } else {
                    assert_eq!(request.target, "/api/files");
                    upload_number += 1;
                    let body = format!(
                        r#"{{"id":{},"fileId":"44444444-4444-4444-8444-444444444444","fileName":"kaka-test.png","fileSize":4}}"#,
                        upload_number
                    );
                    write_json(&mut stream, "201 Created", &body, true);
                }
                server_captured.lock().unwrap().push(request);
            }
        });

        let directory = tempfile::tempdir().unwrap();
        let config = directory.path().join(CONFIG_FILENAME);
        save_preferences(
            &config,
            &SharePreferences {
                origin: Some(format!("http://127.0.0.1:{port}")),
                identity: None,
            },
        )
        .unwrap();

        let first = share_image(&config, "截图 2026.png", b"png1")
            .await
            .unwrap();
        let second = share_image(&config, "截图 2026.png", b"png2")
            .await
            .unwrap();
        server.join().unwrap();
        assert_eq!((first.message_id, second.message_id), (1, 2));

        let requests = captured.lock().unwrap();
        let uploads = requests
            .iter()
            .filter(|request| request.target == "/api/files")
            .collect::<Vec<_>>();
        assert_eq!(uploads.len(), 2);
        assert!(uploads[0]
            .headers
            .to_ascii_lowercase()
            .contains("x-file-name: %e6%88%aa%e5%9b%be%202026%2epng"));
        assert_eq!(uploads[0].body, b"png1");
        assert!(uploads[1]
            .headers
            .to_ascii_lowercase()
            .contains("cookie: lan_identity=11111111-1111-4111-8111-111111111111"));
        let preferences = load_preferences(&config);
        assert_eq!(
            preferences.identity.unwrap().cookie,
            "lan_identity=11111111-1111-4111-8111-111111111111"
        );
    }

    #[tokio::test]
    #[ignore = "requires LANCHAT_E2E_ORIGIN pointing at a running LanChat"]
    async fn shares_to_a_running_lanchat_and_finds_the_message() {
        let origin = std::env::var("LANCHAT_E2E_ORIGIN").unwrap();
        let directory = tempfile::tempdir().unwrap();
        let config = directory.path().join(CONFIG_FILENAME);
        save_preferences(
            &config,
            &SharePreferences {
                origin: Some(origin.clone()),
                identity: None,
            },
        )
        .unwrap();
        let png = include_bytes!("../tests/fixtures/qr/url.png");
        let result = share_image(&config, "kaka-e2e.png", png).await.unwrap();
        assert_eq!(result.origin, origin);
        assert!(result.message_id > 0);
        assert_eq!(result.file_name, "kaka-e2e.png");
        assert_eq!(result.file_size, png.len() as u64);

        let messages = upload_client()
            .unwrap()
            .get(format!("{origin}/api/messages"))
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        assert!(messages["messages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|message| {
                message["id"].as_i64() == Some(result.message_id)
                    && message["fileName"].as_str() == Some("kaka-e2e.png")
            }));
    }
}
