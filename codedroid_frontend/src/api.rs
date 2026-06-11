pub const DEFAULT_API_URL: &str = "http://localhost:3000";

/// Returns the backend API URL — reads from LocalStorage settings if set,
/// otherwise falls back to DEFAULT_API_URL (localhost:3000).
pub fn get_api_url() -> String {
    use gloo_storage::{LocalStorage, Storage};
    if let Ok(settings) = LocalStorage::get::<crate::models::Settings>("codedroid_settings") {
        let url = settings.api_url.trim().to_string();
        if !url.is_empty() {
            return url;
        }
    }
    DEFAULT_API_URL.to_string()
}

use crate::models::PackageResponse;
use gloo_net::http::Request;
use serde_json::json;

pub async fn save_file_api(path: &str, content: &str) -> Result<(), String> {
    let body = json!({ "path": path, "content": content });
    Request::post(&format!("{}/sync_file", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn add_package(
    package: &str,
    language: &str,
    project_path: &str,
) -> Result<PackageResponse, String> {
    let body = json!({
        "package": package,
        "language": language,
        "project_path": project_path,
    });
    Request::post(&format!("{}/add_package", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<PackageResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize, Clone, PartialEq)]
pub struct CompletionItem {
    pub label: String,
    pub insert_text: Option<String>,
    pub kind: Option<u32>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CompletionResponse {
    pub suggestions: Vec<CompletionItem>,
}

pub async fn get_completions_api(
    code: &str,
    language: &str,
    project_path: &str,
    file_path: &str,
    line: u32,
    character: u32,
) -> Result<CompletionResponse, String> {
    let body = json!({
        "code": code,
        "language": language,
        "project_path": project_path,
        "file_path": file_path,
        "line": line,
        "character": character
    });
    Request::post(&format!("{}/complete", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<CompletionResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default, PartialEq)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default, PartialEq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default, PartialEq)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

#[derive(serde::Deserialize)]
pub struct DefinitionResponse {
    pub locations: Vec<Location>,
}

#[derive(serde::Deserialize)]
pub struct ReferencesResponse {
    pub locations: Vec<Location>,
}

pub async fn get_definition_api(
    code: &str,
    language: &str,
    project_path: &str,
    file_path: &str,
    line: u32,
    character: u32,
) -> Result<DefinitionResponse, String> {
    let body = json!({
        "code": code,
        "language": language,
        "project_path": project_path,
        "file_path": file_path,
        "line": line,
        "character": character
    });
    Request::post(&format!("{}/definition", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<DefinitionResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn get_references_api(
    code: &str,
    language: &str,
    project_path: &str,
    file_path: &str,
    line: u32,
    character: u32,
) -> Result<ReferencesResponse, String> {
    let body = json!({
        "code": code,
        "language": language,
        "project_path": project_path,
        "file_path": file_path,
        "line": line,
        "character": character
    });
    Request::post(&format!("{}/references", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<ReferencesResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn delete_file_api(path: &str, is_dir: bool) -> Result<(), String> {
    let body = json!({ "path": path, "is_dir": is_dir });
    Request::post(&format!("{}/delete_file", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn copy_file_api(src_path: &str, dest_path: &str, is_dir: bool) -> Result<(), String> {
    let body = json!({ "src_path": src_path, "dest_path": dest_path, "is_dir": is_dir });
    Request::post(&format!("{}/copy_file", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn move_file_api(src_path: &str, dest_path: &str) -> Result<(), String> {
    let body = json!({ "src_path": src_path, "dest_path": dest_path });
    Request::post(&format!("{}/move_file", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn create_dir_api(path: &str) -> Result<(), String> {
    let body = json!({ "path": path });
    Request::post(&format!("{}/create_dir", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Option<u32>,
    pub code: Option<serde_json::Value>,
    pub source: Option<String>,
    pub message: String,
    #[serde(default)]
    pub file: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
pub struct DiagnosticsResponse {
    pub diagnostics: Vec<Diagnostic>,
}

pub async fn get_diagnostics_api(
    code: &str,
    language: &str,
    project_path: &str,
    file_path: &str,
) -> Result<DiagnosticsResponse, String> {
    let body = json!({
        "code": code,
        "language": language,
        "project_path": project_path,
        "file_path": file_path,
    });
    Request::post(&format!("{}/diagnostics", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<DiagnosticsResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
pub struct CodeSuggestion {
    pub title: String,
    pub explanation: String,
    pub replacement: Option<String>,
    pub range: Option<Range>,
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct SuggestionResponse {
    pub suggestions: Vec<CodeSuggestion>,
}

pub async fn get_error_suggestions_api(
    code: &str,
    language: &str,
    diagnostic: &Diagnostic,
) -> Result<SuggestionResponse, String> {
    let body = json!({
        "code": code,
        "language": language,
        "diagnostic": diagnostic,
    });
    Request::post(&format!("{}/error_suggestions", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<SuggestionResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize, Clone, PartialEq)]
pub struct FormatResponse {
    pub formatted_code: String,
    pub error: Option<String>,
}

pub async fn format_code_api(
    code: &str,
    language: &str,
    project_path: &str,
) -> Result<FormatResponse, String> {
    let body = json!({
        "code": code,
        "language": language,
        "project_path": project_path,
    });
    Request::post(&format!("{}/format", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<FormatResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct RunCodeResponse {
    pub output: String,
    pub error: String,
    pub pid: Option<u32>,
    pub url: Option<String>,
}

pub async fn run_code(
    code: &str,
    language: &str,
    project_path: &str,
    file_path: Option<&str>,
) -> Result<RunCodeResponse, String> {
    let body = json!({
        "code": code,
        "language": language,
        "project_path": project_path,
        "file_path": file_path,
    });
    Request::post(&format!("{}/run", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<RunCodeResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn stop_process(
    pid: Option<u32>,
    stop_live_server: bool,
) -> Result<RunCodeResponse, String> {
    let body = json!({
        "pid": pid,
        "stop_live_server": stop_live_server,
    });
    Request::post(&format!("{}/stop", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<RunCodeResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct ReadDocRequest {
    pub path: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct ReadDocResponse {
    pub content: String,
    pub error: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct ListDocsResponse {
    pub files: Vec<String>,
    pub error: String,
}

pub async fn read_doc_api(path: &str) -> Result<ReadDocResponse, String> {
    let body = ReadDocRequest {
        path: path.to_string(),
    };
    Request::post(&format!("{}/docs/read", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<ReadDocResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn list_docs_api() -> Result<ListDocsResponse, String> {
    Request::get(&format!("{}/docs/list", get_api_url()))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<ListDocsResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct ReadFileRequest {
    pub path: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct ReadFileResponse {
    pub content: String,
    pub error: String,
}

pub async fn read_file_api(path: &str) -> Result<ReadFileResponse, String> {
    let body = ReadFileRequest {
        path: path.to_string(),
    };
    Request::post(&format!("{}/read_file", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<ReadFileResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct HoverRequest {
    pub project_path: String,
    pub file_path: String,
    pub code: String,
    pub line: u32,
    pub character: u32,
    pub language: String,
}

#[derive(serde::Deserialize, Clone)]
#[allow(dead_code)]
pub struct HoverResponse {
    pub contents: Option<String>,
    pub error: String,
}

pub async fn hover_api(
    project_path: &str,
    file_path: &str,
    code: &str,
    line: u32,
    character: u32,
    language: &str,
) -> Result<HoverResponse, String> {
    let body = HoverRequest {
        project_path: project_path.to_string(),
        file_path: file_path.to_string(),
        code: code.to_string(),
        line,
        character,
        language: language.to_string(),
    };
    Request::post(&format!("{}/hover", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<HoverResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct CommandRequest {
    pub command: String,
    pub project_path: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct CommandResponse {
    pub output: String,
    pub error: String,
    pub success: bool,
    #[allow(dead_code)]
    pub pid: Option<u32>,
}

pub async fn run_command_api(project_path: &str, command: &str) -> Result<CommandResponse, String> {
    let body = CommandRequest {
        project_path: project_path.to_string(),
        command: command.to_string(),
    };
    Request::post(&format!("{}/run_command", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<CommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct ScanProjectRequest {
    pub project_path: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct FileInfo {
    pub rel_path: String,
    pub is_dir: bool,
}

#[derive(serde::Deserialize, Clone)]
pub struct ScanProjectResponse {
    pub files: Vec<FileInfo>,
    pub error: String,
}

pub async fn scan_project_api(project_path: &str) -> Result<ScanProjectResponse, String> {
    let body = ScanProjectRequest {
        project_path: project_path.to_string(),
    };
    Request::post(&format!("{}/scan_project", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<ScanProjectResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize)]
struct StartTerminalResponse {
    session_id: String,
}

#[derive(serde::Deserialize)]
struct TerminalOutputResponse {
    output: String,
    is_alive: bool,
}

#[derive(serde::Deserialize)]
struct TerminalInputResponse {
    success: bool,
    #[allow(dead_code)]
    error: Option<String>,
}

#[derive(serde::Deserialize)]
struct StopTerminalResponse {
    success: bool,
}

pub async fn start_terminal_api(project_path: &str) -> Result<String, String> {
    let body = json!({ "project_path": project_path });
    Request::post(&format!("{}/terminal/start", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<StartTerminalResponse>()
        .await
        .map(move |r| r.session_id)
        .map_err(|e| e.to_string())
}

pub async fn poll_terminal_output_api(session_id: &str) -> Result<(String, bool), String> {
    let body = json!({ "session_id": session_id });
    Request::post(&format!("{}/terminal/output", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<TerminalOutputResponse>()
        .await
        .map(move |r| (r.output, r.is_alive))
        .map_err(|e| e.to_string())
}

pub async fn send_terminal_input_api(session_id: &str, input: &str) -> Result<bool, String> {
    let body = json!({ "session_id": session_id, "input": input });
    Request::post(&format!("{}/terminal/input", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<TerminalInputResponse>()
        .await
        .map(move |r| r.success)
        .map_err(|e| e.to_string())
}

pub async fn stop_terminal_api(session_id: &str) -> Result<bool, String> {
    let body = json!({ "session_id": session_id });
    Request::post(&format!("{}/terminal/stop", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<StopTerminalResponse>()
        .await
        .map(move |r| r.success)
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct GitRequest {
    pub project_path: String,
}

#[derive(serde::Serialize)]
pub struct GitFileRequest {
    pub project_path: String,
    pub file_path: String,
}

#[derive(serde::Serialize)]
pub struct GitCommitRequest {
    pub project_path: String,
    pub message: String,
}

#[derive(serde::Serialize)]
pub struct GitCloneRequest {
    pub clone_url: String,
    pub project_name: String,
    pub project_path: Option<String>,
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct GitStatusFile {
    pub path: String,
    pub status: String,
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct GitStatusResponse {
    pub branch: String,
    pub files: Vec<GitStatusFile>,
    pub error: Option<String>,
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct GitCommandResponse {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct GitDiffLinesResponse {
    pub added: Vec<usize>,
    pub modified: Vec<usize>,
    pub deleted: Vec<usize>,
}

pub async fn git_status_api(project_path: &str) -> Result<GitStatusResponse, String> {
    let body = GitRequest { project_path: project_path.to_string() };
    Request::post(&format!("{}/git/status", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitStatusResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_stage_api(project_path: &str, file_path: &str) -> Result<GitCommandResponse, String> {
    let body = GitFileRequest { project_path: project_path.to_string(), file_path: file_path.to_string() };
    Request::post(&format!("{}/git/stage", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_unstage_api(project_path: &str, file_path: &str) -> Result<GitCommandResponse, String> {
    let body = GitFileRequest { project_path: project_path.to_string(), file_path: file_path.to_string() };
    Request::post(&format!("{}/git/unstage", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_discard_api(project_path: &str, file_path: &str) -> Result<GitCommandResponse, String> {
    let body = GitFileRequest { project_path: project_path.to_string(), file_path: file_path.to_string() };
    Request::post(&format!("{}/git/discard", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_stage_all_api(project_path: &str) -> Result<GitCommandResponse, String> {
    let body = GitRequest { project_path: project_path.to_string() };
    Request::post(&format!("{}/git/stage-all", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_unstage_all_api(project_path: &str) -> Result<GitCommandResponse, String> {
    let body = GitRequest { project_path: project_path.to_string() };
    Request::post(&format!("{}/git/unstage-all", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_discard_all_api(project_path: &str) -> Result<GitCommandResponse, String> {
    let body = GitRequest { project_path: project_path.to_string() };
    Request::post(&format!("{}/git/discard-all", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_commit_api(project_path: &str, message: &str) -> Result<GitCommandResponse, String> {
    let body = GitCommitRequest { project_path: project_path.to_string(), message: message.to_string() };
    Request::post(&format!("{}/git/commit", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_push_api(project_path: &str) -> Result<GitCommandResponse, String> {
    let body = GitRequest { project_path: project_path.to_string() };
    Request::post(&format!("{}/git/push", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_pull_api(project_path: &str) -> Result<GitCommandResponse, String> {
    let body = GitRequest { project_path: project_path.to_string() };
    Request::post(&format!("{}/git/pull", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_diff_lines_api(project_path: &str, file_path: &str) -> Result<GitDiffLinesResponse, String> {
    let body = GitFileRequest { project_path: project_path.to_string(), file_path: file_path.to_string() };
    Request::post(&format!("{}/git/diff_lines", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitDiffLinesResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_diff_text_api(project_path: &str, file_path: &str) -> Result<GitCommandResponse, String> {
    let body = GitFileRequest { project_path: project_path.to_string(), file_path: file_path.to_string() };
    Request::post(&format!("{}/git/diff_text", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_clone_api(clone_url: &str, project_name: &str, project_path: &str) -> Result<GitCommandResponse, String> {
    let body = GitCloneRequest {
        clone_url: clone_url.to_string(),
        project_name: project_name.to_string(),
        project_path: Some(project_path.to_string()),
    };
    Request::post(&format!("{}/git/clone", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct GitCommitInfo {
    pub hash: String,
    pub subject: String,
    pub refs: String,
    pub author_name: String,
    pub relative_date: String,
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct GitLogResponse {
    pub commits: Vec<GitCommitInfo>,
    pub error: Option<String>,
}

pub async fn git_log_api(project_path: &str) -> Result<GitLogResponse, String> {
    let body = GitRequest { project_path: project_path.to_string() };
    Request::post(&format!("{}/git/log", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitLogResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct GitBranchRequest {
    pub project_path: String,
    pub branch_name: String,
    pub start_point: Option<String>,
}

#[derive(serde::Serialize)]
pub struct GitMergeRequest {
    pub project_path: String,
    pub branch_name: String,
}

#[derive(serde::Serialize)]
pub struct GitRemoteRequest {
    pub project_path: String,
    pub remote_name: String,
    pub remote_url: Option<String>,
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct GitBranchesResponse {
    pub current: String,
    pub local: Vec<String>,
    pub remote: Vec<String>,
    pub error: Option<String>,
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct GitRemoteInfo {
    pub name: String,
    pub url: String,
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct GitRemotesResponse {
    pub remotes: Vec<GitRemoteInfo>,
    pub error: Option<String>,
}

pub async fn git_list_branches_api(project_path: &str) -> Result<GitBranchesResponse, String> {
    let body = GitRequest { project_path: project_path.to_string() };
    Request::post(&format!("{}/git/branches", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitBranchesResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_create_branch_api(
    project_path: &str,
    branch_name: &str,
    start_point: Option<&str>,
) -> Result<GitCommandResponse, String> {
    let body = GitBranchRequest {
        project_path: project_path.to_string(),
        branch_name: branch_name.to_string(),
        start_point: start_point.map(|s| s.to_string()),
    };
    Request::post(&format!("{}/git/branch/create", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_checkout_branch_api(
    project_path: &str,
    branch_name: &str,
) -> Result<GitCommandResponse, String> {
    let body = GitBranchRequest {
        project_path: project_path.to_string(),
        branch_name: branch_name.to_string(),
        start_point: None,
    };
    Request::post(&format!("{}/git/branch/checkout", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_merge_branch_api(
    project_path: &str,
    branch_name: &str,
) -> Result<GitCommandResponse, String> {
    let body = GitMergeRequest {
        project_path: project_path.to_string(),
        branch_name: branch_name.to_string(),
    };
    Request::post(&format!("{}/git/branch/merge", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_list_remotes_api(project_path: &str) -> Result<GitRemotesResponse, String> {
    let body = GitRequest { project_path: project_path.to_string() };
    Request::post(&format!("{}/git/remotes", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitRemotesResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_add_remote_api(
    project_path: &str,
    remote_name: &str,
    remote_url: &str,
) -> Result<GitCommandResponse, String> {
    let body = GitRemoteRequest {
        project_path: project_path.to_string(),
        remote_name: remote_name.to_string(),
        remote_url: Some(remote_url.to_string()),
    };
    Request::post(&format!("{}/git/remote/add", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_remove_remote_api(
    project_path: &str,
    remote_name: &str,
) -> Result<GitCommandResponse, String> {
    let body = GitRemoteRequest {
        project_path: project_path.to_string(),
        remote_name: remote_name.to_string(),
        remote_url: None,
    };
    Request::post(&format!("{}/git/remote/remove", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn git_set_remote_url_api(
    project_path: &str,
    remote_name: &str,
    remote_url: &str,
) -> Result<GitCommandResponse, String> {
    let body = GitRemoteRequest {
        project_path: project_path.to_string(),
        remote_name: remote_name.to_string(),
        remote_url: Some(remote_url.to_string()),
    };
    Request::post(&format!("{}/git/remote/set-url", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<GitCommandResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct PickDirectoryResponse {
    pub success: bool,
    pub path: Option<String>,
    pub error: Option<String>,
}

pub async fn pick_directory_api() -> Result<PickDirectoryResponse, String> {
    Request::post(&format!("{}/pick_directory", get_api_url()))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<PickDirectoryResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize, Clone)]
pub struct CreateProjectRequest {
    pub name: String,
    pub language: String,
    pub framework: String,
    pub path: String,
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct CreateProjectResponse {
    pub success: bool,
    pub error: String,
}

pub async fn create_project_api(req: CreateProjectRequest) -> Result<CreateProjectResponse, String> {
    Request::post(&format!("{}/create_project", get_api_url()))
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<CreateProjectResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct LiveServerStatusResponse {
    pub running: bool,
    pub port: Option<u16>,
    pub project_path: Option<String>,
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct LiveServerStopResponse {
    pub success: bool,
}

#[derive(serde::Deserialize, Clone, PartialEq, Debug)]
pub struct LiveServerStartResponse {
    pub port: u16,
}

pub async fn start_live_server_api(project_path: &str) -> Result<LiveServerStartResponse, String> {
    let body = serde_json::json!({ "project_path": project_path });
    Request::post(&format!("{}/live-server/start", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<LiveServerStartResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn stop_live_server_api() -> Result<LiveServerStopResponse, String> {
    Request::post(&format!("{}/live-server/stop", get_api_url()))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<LiveServerStopResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn get_live_server_status_api() -> Result<LiveServerStatusResponse, String> {
    Request::get(&format!("{}/live-server/status", get_api_url()))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<LiveServerStatusResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
pub struct DocumentSymbolResponse {
    pub name: String,
    pub kind: u32,
    pub line: u32,
    pub character: u32,
    pub container_name: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct SymbolsResponse {
    pub symbols: Vec<DocumentSymbolResponse>,
}

pub async fn get_symbols_api(
    code: &str,
    language: &str,
    project_path: &str,
    file_path: Option<&str>,
) -> Result<SymbolsResponse, String> {
    let body = serde_json::json!({
        "code": code,
        "language": language,
        "project_path": project_path,
        "file_path": file_path,
    });
    Request::post(&format!("{}/symbols", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<SymbolsResponse>()
        .await
        .map_err(|e| e.to_string())
}


