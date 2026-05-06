use serde::{Deserialize, Serialize};
use crate::lsp;

#[derive(Deserialize)]
pub struct CodeRequest {
    pub code: String,
    pub language: String,
    pub project_path: String,
    pub cargo_toml: Option<String>,
}

#[derive(Serialize)]
pub struct CodeResponse {
    pub output: String,
    pub error: String,
    pub pid: Option<u32>,
    pub url: Option<String>,
}

#[derive(Deserialize)]
pub struct StopRequest {
    pub pid: u32,
}

#[derive(Deserialize)]
pub struct PackageRequest {
    pub package: String,
    pub language: String,
    pub project_path: String,
}

#[derive(Deserialize)]
pub struct SyncRequest {
    pub path: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct CompletionRequest {
    pub code: String,
    pub language: String,
    pub project_path: String,
    pub line: u32,
    pub character: u32,
}

#[derive(Serialize)]
pub struct CompletionResponse {
    pub suggestions: Vec<lsp::CompletionItem>,
}
