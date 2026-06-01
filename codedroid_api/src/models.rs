use crate::lsp;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
pub struct PackageResponse {
    pub output: String,
    pub error: String,
    pub dependency_file_name: Option<String>,
    pub dependency_file_content: Option<String>,
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
    pub file_path: Option<String>,
    pub line: u32,
    pub character: u32,
}

#[derive(Serialize)]
pub struct CompletionResponse {
    pub suggestions: Vec<lsp::CompletionItem>,
}

#[derive(Deserialize)]
pub struct DeleteRequest {
    pub path: String,
    pub is_dir: bool,
}

#[derive(Deserialize)]
pub struct CopyRequest {
    pub src_path: String,
    pub dest_path: String,
    pub is_dir: bool,
}

#[derive(Deserialize)]
pub struct MoveRequest {
    pub src_path: String,
    pub dest_path: String,
}

#[derive(Deserialize)]
pub struct CreateDirRequest {
    pub path: String,
}

#[derive(Deserialize)]
pub struct FormatRequest {
    pub code: String,
    pub language: String,
    pub project_path: String,
}

#[derive(Serialize)]
pub struct FormatResponse {
    pub formatted_code: String,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct DefinitionRequest {
    pub code: String,
    pub language: String,
    pub project_path: String,
    pub file_path: Option<String>,
    pub line: u32,
    pub character: u32,
}

#[derive(Serialize)]
pub struct DefinitionResponse {
    pub locations: Vec<lsp::Location>,
}

#[derive(Deserialize)]
pub struct ReferencesRequest {
    pub code: String,
    pub language: String,
    pub project_path: String,
    pub file_path: Option<String>,
    pub line: u32,
    pub character: u32,
}

#[derive(Serialize)]
pub struct ReferencesResponse {
    pub locations: Vec<lsp::Location>,
}

#[derive(Deserialize)]
pub struct ReadFileRequest {
    pub path: String,
}

#[derive(Serialize)]
pub struct ReadFileResponse {
    pub content: String,
    pub error: String,
}

#[derive(Deserialize)]
pub struct HoverRequest {
    pub project_path: String,
    pub file_path: String,
    pub code: String,
    pub line: u32,
    pub character: u32,
    pub language: String,
}

#[derive(Serialize)]
pub struct HoverResponse {
    pub contents: Option<String>,
    pub error: String,
}

#[derive(Deserialize)]
pub struct CommandRequest {
    pub command: String,
    pub project_path: String,
}

#[derive(Serialize)]
pub struct CommandResponse {
    pub output: String,
    pub error: String,
    pub success: bool,
    pub pid: Option<u32>,
}

#[derive(Deserialize)]
pub struct ScanProjectRequest {
    pub project_path: String,
}

#[derive(Serialize, Clone)]
pub struct FileInfo {
    pub rel_path: String,
    pub is_dir: bool,
}

#[derive(Serialize)]
pub struct ScanProjectResponse {
    pub files: Vec<FileInfo>,
    pub error: String,
}

#[derive(Serialize)]
pub struct PickDirectoryResponse {
    pub success: bool,
    pub path: Option<String>,
    pub error: Option<String>,
}
