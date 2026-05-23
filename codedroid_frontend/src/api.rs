use gloo_net::http::Request;
use serde_json::json;
use crate::models::RunResponse;

pub const API_URL: &str = "http://localhost:3000";

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

    Request::post(&format!("{}/run", API_URL))
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
    Request::post(&format!("{}/stop", API_URL))
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
    Request::post(&format!("{}/sync_file", API_URL))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn add_package(package: &str, language: &str, project_path: &str) -> Result<RunResponse, String> {
    let body = json!({
        "package": package,
        "language": language,
        "project_path": project_path,
    });
    Request::post(&format!("{}/add_package", API_URL))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<RunResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize, Clone, PartialEq)]
pub struct CompletionItem {
    pub label: String,
    pub kind: Option<u32>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CompletionResponse {
    pub suggestions: Vec<CompletionItem>,
}

pub async fn get_completions_api(code: &str, language: &str, project_path: &str, line: u32, character: u32) -> Result<CompletionResponse, String> {
    let body = json!({
        "code": code,
        "language": language,
        "project_path": project_path,
        "line": line,
        "character": character
    });
    Request::post(&format!("{}/complete", API_URL))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<CompletionResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn delete_file_api(path: &str, is_dir: bool) -> Result<(), String> {
    let body = json!({ "path": path, "is_dir": is_dir });
    Request::post(&format!("{}/delete_file", API_URL))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn copy_file_api(src_path: &str, dest_path: &str, is_dir: bool) -> Result<(), String> {
    let body = json!({ "src_path": src_path, "dest_path": dest_path, "is_dir": is_dir });
    Request::post(&format!("{}/copy_file", API_URL))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn create_dir_api(path: &str) -> Result<(), String> {
    let body = json!({ "path": path });
    Request::post(&format!("{}/create_dir", API_URL))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

