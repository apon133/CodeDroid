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

use gloo_net::http::Request;
use serde_json::json;
use crate::models::{RunResponse, PackageResponse, CommandResponse};

pub async fn run_code(
    code: &str,
    language: &str,
    project_path: &str,
    cargo_toml: Option<&str>,
) -> Result<RunResponse, String> {
    let body = json!({
        "code": code,
        "language": language,
        "project_path": project_path,
        "cargo_toml": cargo_toml,
    });

    Request::post(&format!("{}/run", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<RunResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn stop_process(pid: u32) -> Result<RunResponse, String> {
    let body = json!({ "pid": pid });
    Request::post(&format!("{}/stop", get_api_url()))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<RunResponse>()
        .await
        .map_err(|e| e.to_string())
}

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

pub async fn add_package(package: &str, language: &str, project_path: &str) -> Result<PackageResponse, String> {
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

pub async fn get_completions_api(code: &str, language: &str, project_path: &str, file_path: &str, line: u32, character: u32) -> Result<CompletionResponse, String> {
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

pub async fn get_diagnostics_api(code: &str, language: &str, project_path: &str, file_path: &str) -> Result<DiagnosticsResponse, String> {
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
    let body = ReadDocRequest { path: path.to_string() };
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
    let body = ReadFileRequest { path: path.to_string() };
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

pub async fn run_command_api(
    command: &str,
    project_path: &str,
) -> Result<CommandResponse, String> {
    let body = json!({
        "command": command,
        "project_path": project_path,
    });
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



