use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Deserialize)]
pub struct StartRequest {
    pub project_path: String,
}

#[derive(Debug, Serialize)]
pub struct StartResponse {
    pub port: u16,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub running: bool,
    pub port: Option<u16>,
    pub project_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StopResponse {
    pub success: bool,
}

struct LiveServerState {
    project_path: String,
    port: u16,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    watcher_shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

#[derive(Clone)]
struct RouterState {
    project_dir: PathBuf,
    reload_rx: std::sync::Arc<std::sync::Mutex<tokio::sync::watch::Receiver<u64>>>,
}

fn get_latest_modified_time(
    dir: &std::path::Path,
) -> Option<(std::time::SystemTime, std::path::PathBuf)> {
    let mut latest = None;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name != "node_modules"
                    && name != "target"
                    && name != ".git"
                    && name != "build"
                    && name != "dist"
                    && name != ".planning"
                    && name != ".gemini"
                {
                    if let Some((mod_time, mod_path)) = get_latest_modified_time(&path) {
                        if latest
                            .as_ref()
                            .map_or(true, |(l_time, _)| mod_time > *l_time)
                        {
                            latest = Some((mod_time, mod_path));
                        }
                    }
                }
            } else if path.is_file() {
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if let Ok(mod_time) = metadata.modified() {
                        if latest
                            .as_ref()
                            .map_or(true, |(l_time, _)| mod_time > *l_time)
                        {
                            latest = Some((mod_time, path.clone()));
                        }
                    }
                }
            }
        }
    }
    latest
}

static LIVE_SERVER: OnceLock<Mutex<Option<LiveServerState>>> = OnceLock::new();

fn get_live_server() -> &'static Mutex<Option<LiveServerState>> {
    LIVE_SERVER.get_or_init(|| Mutex::new(None))
}

fn find_free_port(start_port: u16) -> Option<u16> {
    for port in start_port..6000 {
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Some(port);
        }
    }
    None
}

// Handler for the static files served on the dynamic live server port
async fn live_server_handler(
    State(state): State<RouterState>,
    uri: axum::http::Uri,
) -> impl IntoResponse {
    let project_dir = state.project_dir;
    let mut relative_path = uri.path().trim_start_matches('/');
    if relative_path.is_empty() {
        relative_path = "index.html";
    }

    let file_path = project_dir.join(relative_path);
    println!(
        "[LIVE SERVER] project_dir={:?}, file_path={:?}, uri={:?}",
        project_dir, file_path, uri
    );

    // Security: canonicalize and verify path is inside project_dir
    let canonical_project_dir = match std::fs::canonicalize(&project_dir) {
        Ok(p) => p,
        Err(e) => {
            println!(
                "[LIVE SERVER] Error canonicalizing project_dir {:?}: {:?}",
                project_dir, e
            );
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid project directory",
            )
                .into_response();
        }
    };

    let canonical_file_path = match std::fs::canonicalize(&file_path) {
        Ok(p) => p,
        Err(e) => {
            println!(
                "[LIVE SERVER] Error canonicalizing file_path {:?}: {:?}",
                file_path, e
            );
            return (StatusCode::NOT_FOUND, "File not found").into_response();
        }
    };

    if !canonical_file_path.starts_with(&canonical_project_dir) {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }

    match std::fs::read(&canonical_file_path) {
        Ok(content) => {
            let extension = canonical_file_path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");

            let mime_type = match extension.to_lowercase().as_str() {
                "html" | "htm" => "text/html",
                "css" => "text/css",
                "js" => "application/javascript",
                "json" => "application/json",
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "webp" => "image/webp",
                "svg" => "image/svg+xml",
                "mp3" => "audio/mpeg",
                "wav" => "audio/wav",
                "mp4" => "video/mp4",
                _ => "application/octet-stream",
            };

            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, mime_type.parse().unwrap());
            headers.insert(
                header::CACHE_CONTROL,
                "no-store, no-cache, must-revalidate, max-age=0"
                    .parse()
                    .unwrap(),
            );
            headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());

            let ext_lower = extension.to_lowercase();
            if ext_lower == "html" || ext_lower == "htm" {
                let mut html_str = String::from_utf8_lossy(&content).to_string();
                let current_version = *state.reload_rx.lock().unwrap().borrow();
                let script = format!(
                    r#"
<script>
    (function() {{
        console.log("CodeDroid Live Reload active.");
        let currentVersion = {};
        async function poll() {{
            try {{
                let res = await fetch('/__live_reload_poll?v=' + currentVersion);
                if (res.status === 200) {{
                    let data = await res.json();
                    if (data.reload) {{
                        console.log("File change detected. Reloading...");
                        window.location.reload();
                        return;
                    }}
                }}
            }} catch (e) {{
                console.error("Live reload error:", e);
            }}
            setTimeout(poll, 1000);
        }}
        poll();
    }})();
</script>
"#,
                    current_version
                );
                if let Some(pos) = html_str.rfind("</body>") {
                    html_str.insert_str(pos, &script);
                } else {
                    html_str.push_str(&script);
                }
                (StatusCode::OK, headers, html_str.into_bytes()).into_response()
            } else {
                (StatusCode::OK, headers, content).into_response()
            }
        }
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

#[derive(serde::Deserialize)]
struct PollParams {
    v: Option<u64>,
}

async fn live_reload_poll(
    State(state): State<RouterState>,
    Query(params): Query<PollParams>,
) -> impl IntoResponse {
    let client_version = params.v.unwrap_or(0);
    let mut rx = {
        let mut rx_guard = state.reload_rx.lock().unwrap();
        let _ = rx_guard.borrow_and_update();
        rx_guard.clone()
    };
    let current_version = *rx.borrow();

    if current_version > client_version {
        return (StatusCode::OK, Json(serde_json::json!({ "reload": true }))).into_response();
    }

    let result = tokio::time::timeout(tokio::time::Duration::from_secs(30), rx.changed()).await;
    let reload = result.is_ok();
    (
        StatusCode::OK,
        Json(serde_json::json!({ "reload": reload })),
    )
        .into_response()
}

pub async fn ensure_live_server(project_path: &str) -> Result<u16, String> {
    let mut server_guard = get_live_server().lock().unwrap();

    if let Some(ref current) = *server_guard {
        if current.project_path == project_path {
            return Ok(current.port);
        }
        if let Some(shutdown_tx) = server_guard.as_mut().and_then(|c| c.shutdown_tx.take()) {
            let _ = shutdown_tx.send(());
        }
        if let Some(watcher_shutdown_tx) = server_guard
            .as_mut()
            .and_then(|c| c.watcher_shutdown_tx.take())
        {
            let _ = watcher_shutdown_tx.send(());
        }
        *server_guard = None;
    }

    let port = find_free_port(5500).ok_or_else(|| "No free ports available".to_string())?;

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let (watcher_shutdown_tx, mut watcher_shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let (reload_tx, reload_rx) = tokio::sync::watch::channel(0u64);

    let resolved_path = crate::utils::resolve_project_dir(project_path);
    let project_dir_buf = PathBuf::from(resolved_path);
    let project_path_owned = project_path.to_string();

    let router_state = RouterState {
        project_dir: project_dir_buf.clone(),
        reload_rx: std::sync::Arc::new(std::sync::Mutex::new(reload_rx)),
    };

    let app = Router::new()
        .route("/__live_reload_poll", get(live_reload_poll))
        .fallback(live_server_handler)
        .with_state(router_state)
        .layer(tower_http::cors::CorsLayer::permissive());

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));

    // Spawn server
    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(_) => return,
        };

        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await;
    });

    // Spawn watcher
    let project_dir_clone = project_dir_buf.clone();
    tokio::spawn(async move {
        let mut last_modified = get_latest_modified_time(&project_dir_clone);
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let current_modified = get_latest_modified_time(&project_dir_clone);
                    if let Some((curr_time, ref curr_path)) = current_modified {
                        let is_newer = match last_modified {
                            Some((last_time, _)) => curr_time > last_time,
                            None => true,
                        };
                        if is_newer {
                            last_modified = current_modified.clone();
                            println!("[LIVE SERVER] File change detected in {:?}! Triggering reload...", curr_path);
                            let now_epoch = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            let _ = reload_tx.send(now_epoch);
                        }
                    }
                }
                _ = &mut watcher_shutdown_rx => {
                    println!("[LIVE SERVER] Folder watcher shutdown received.");
                    break;
                }
            }
        }
    });

    *server_guard = Some(LiveServerState {
        project_path: project_path_owned,
        port,
        shutdown_tx: Some(shutdown_tx),
        watcher_shutdown_tx: Some(watcher_shutdown_tx),
    });

    Ok(port)
}

pub async fn start_live_server(Json(payload): Json<StartRequest>) -> impl IntoResponse {
    println!(
        "[LIVE SERVER] Received start request for project_path={:?}",
        payload.project_path
    );
    match ensure_live_server(&payload.project_path).await {
        Ok(port) => (StatusCode::OK, Json(StartResponse { port })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub fn stop_live_server_internal() {
    let mut server_guard = get_live_server().lock().unwrap();
    if let Some(mut current) = server_guard.take() {
        if let Some(shutdown_tx) = current.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
        if let Some(watcher_shutdown_tx) = current.watcher_shutdown_tx.take() {
            let _ = watcher_shutdown_tx.send(());
        }
    }
}

pub async fn stop_live_server() -> impl IntoResponse {
    stop_live_server_internal();
    (StatusCode::OK, Json(StopResponse { success: true }))
}

pub async fn get_live_server_status() -> impl IntoResponse {
    let server_guard = get_live_server().lock().unwrap();
    if let Some(ref current) = *server_guard {
        Json(StatusResponse {
            running: true,
            port: Some(current.port),
            project_path: Some(current.project_path.clone()),
        })
    } else {
        Json(StatusResponse {
            running: false,
            port: None,
            project_path: None,
        })
    }
}

pub fn router() -> Router {
    Router::new()
        .route("/start", post(start_live_server))
        .route("/stop", post(stop_live_server))
        .route("/status", get(get_live_server_status))
}
