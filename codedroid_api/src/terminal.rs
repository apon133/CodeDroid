use std::collections::HashMap;
use std::io::{Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use crate::utils::resolve_project_dir;

pub struct TerminalSession {
    stdin: ChildStdin,
    output_buffer: Arc<Mutex<String>>,
    child: Arc<Mutex<Child>>,
    last_activity: Instant,
}

static TERMINAL_SESSIONS: OnceLock<Arc<Mutex<HashMap<String, TerminalSession>>>> = OnceLock::new();

fn get_sessions() -> &'static Arc<Mutex<HashMap<String, TerminalSession>>> {
    TERMINAL_SESSIONS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

fn cleanup_inactive_sessions(sessions: &mut HashMap<String, TerminalSession>) {
    let now = Instant::now();
    sessions.retain(|_id, session| {
        let is_alive = match session.child.lock().unwrap().try_wait() {
            Ok(None) => true,
            _ => false,
        };
        let is_recent = now.duration_since(session.last_activity) < Duration::from_secs(300);

        if !is_alive || !is_recent {
            let _ = session.child.lock().unwrap().kill();
            false
        } else {
            true
        }
    });
}

#[derive(Deserialize)]
pub struct StartTerminalRequest {
    pub project_path: String,
}

#[derive(Serialize)]
pub struct StartTerminalResponse {
    pub session_id: String,
}

pub async fn start_terminal(Json(payload): Json<StartTerminalRequest>) -> Json<StartTerminalResponse> {
    let project_dir = resolve_project_dir(&payload.project_path);
    let sessions_arc = get_sessions();
    let mut sessions = sessions_arc.lock().unwrap();

    // Clean up stale sessions
    cleanup_inactive_sessions(&mut sessions);

    let (shell, args): (&str, Vec<&str>) = if cfg!(windows) {
        ("cmd.exe", vec![])
    } else if cfg!(target_os = "macos") {
        ("zsh", vec!["-l"])
    } else {
        ("sh", vec![])
    };

    let mut child = Command::new(shell)
        .args(&args)
        .current_dir(&project_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start terminal shell");

    let stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let output_buffer = Arc::new(Mutex::new(String::new()));

    let out_buf = output_buffer.clone();
    thread::spawn(move || {
        let mut reader = stdout;
        let mut buf = [0; 1024];
        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                break;
            }
            let s = String::from_utf8_lossy(&buf[..n]);
            out_buf.lock().unwrap().push_str(&s);
        }
    });

    let err_buf = output_buffer.clone();
    thread::spawn(move || {
        let mut reader = stderr;
        let mut buf = [0; 1024];
        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                break;
            }
            let s = String::from_utf8_lossy(&buf[..n]);
            err_buf.lock().unwrap().push_str(&s);
        }
    });

    let session_id = format!(
        "term_{}",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
    );

    sessions.insert(
        session_id.clone(),
        TerminalSession {
            stdin,
            output_buffer,
            child: Arc::new(Mutex::new(child)),
            last_activity: Instant::now(),
        },
    );

    Json(StartTerminalResponse { session_id })
}

#[derive(Deserialize)]
pub struct TerminalOutputRequest {
    pub session_id: String,
}

#[derive(Serialize)]
pub struct TerminalOutputResponse {
    pub output: String,
    pub is_alive: bool,
}

pub async fn get_terminal_output(Json(payload): Json<TerminalOutputRequest>) -> Json<TerminalOutputResponse> {
    let sessions_arc = get_sessions();
    let mut sessions = sessions_arc.lock().unwrap();

    let mut output = String::new();
    let mut is_alive = false;

    if let Some(session) = sessions.get_mut(&payload.session_id) {
        session.last_activity = Instant::now();
        let mut buf = session.output_buffer.lock().unwrap();
        output = std::mem::take(&mut *buf);

        let status = session.child.lock().unwrap().try_wait();
        println!("🔍 [DEBUG] Session ID {} try_wait status: {:?}", payload.session_id, status);
        if let Ok(None) = status {
            is_alive = true;
        }
    }

    Json(TerminalOutputResponse { output, is_alive })
}

#[derive(Deserialize)]
pub struct TerminalInputRequest {
    pub session_id: String,
    pub input: String,
}

#[derive(Serialize)]
pub struct TerminalInputResponse {
    pub success: bool,
    pub error: Option<String>,
}

pub async fn send_terminal_input(Json(payload): Json<TerminalInputRequest>) -> Json<TerminalInputResponse> {
    let sessions_arc = get_sessions();
    let mut sessions = sessions_arc.lock().unwrap();

    let mut success = false;
    let mut error = None;

    if let Some(session) = sessions.get_mut(&payload.session_id) {
        session.last_activity = Instant::now();
        match session.stdin.write_all(payload.input.as_bytes()) {
            Ok(_) => {
                let _ = session.stdin.flush();
                success = true;
            }
            Err(e) => {
                error = Some(e.to_string());
            }
        }
    } else {
        error = Some("Session not found".to_string());
    }

    Json(TerminalInputResponse { success, error })
}

#[derive(Deserialize)]
pub struct StopTerminalRequest {
    pub session_id: String,
}

#[derive(Serialize)]
pub struct StopTerminalResponse {
    pub success: bool,
}

pub async fn stop_terminal(Json(payload): Json<StopTerminalRequest>) -> Json<StopTerminalResponse> {
    let sessions_arc = get_sessions();
    let mut sessions = sessions_arc.lock().unwrap();

    let mut success = false;
    if let Some(session) = sessions.remove(&payload.session_id) {
        let _ = session.child.lock().unwrap().kill();
        success = true;
    }

    Json(StopTerminalResponse { success })
}

pub fn router() -> Router {
    Router::new()
        .route("/start", post(start_terminal))
        .route("/output", post(get_terminal_output))
        .route("/input", post(send_terminal_input))
        .route("/stop", post(stop_terminal))
}
