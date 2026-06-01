use crate::lsp::{self, Diagnostic};
use crate::utils::resolve_project_dir;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Deserialize)]
pub struct DiagnosticsRequest {
    pub code: String,
    pub language: String,
    pub project_path: String,
    pub file_path: Option<String>,
}

#[derive(Serialize)]
pub struct DiagnosticsResponse {
    pub diagnostics: Vec<Diagnostic>,
}

pub async fn get_diagnostics_handler(
    Json(payload): Json<DiagnosticsRequest>,
) -> Json<DiagnosticsResponse> {
    let lang = payload.language.to_lowercase();
    let project_dir = resolve_project_dir(&payload.project_path);
    let file_uri = if let Some(ref rel_path) = payload.file_path {
        format!("file://{}/{}", project_dir, rel_path)
    } else {
        match lang.as_str() {
            "rust" => format!("file://{}/src/main.rs", project_dir),
            "python" => format!("file://{}/main.py", project_dir),
            "javascript" => format!("file://{}/main.js", project_dir),
            "typescript" => format!("file://{}/main.ts", project_dir),
            "jsx" => format!("file://{}/main.jsx", project_dir),
            "tsx" => format!("file://{}/main.tsx", project_dir),
            "go" => format!("file://{}/main.go", project_dir),
            "c" => format!("file://{}/main.c", project_dir),
            "cpp" => format!("file://{}/main.cpp", project_dir),
            "java" => format!("file://{}/main.java", project_dir),
            "dart" => format!("file://{}/lib/main.dart", project_dir),
            "ruby" => format!("file://{}/main.rb", project_dir),
            "kotlin" => format!("file://{}/main.kt", project_dir),
            "swift" => format!("file://{}/main.swift", project_dir),
            "html" => format!("file://{}/index.html", project_dir),
            "css" => format!("file://{}/style.css", project_dir),
            "vue" => format!("file://{}/Component.vue", project_dir),
            "svelte" => format!("file://{}/Component.svelte", project_dir),
            _ => format!("file://{}/main.txt", project_dir),
        }
    };

    let jdtls_data = format!("{}/.jdtls_data", project_dir);
    let lsp_cmd = match lang.as_str() {
        "rust" => Some(("rust-analyzer", vec![])),
        "python" => Some(("pylsp", vec![])),
        "javascript" | "typescript" | "jsx" | "tsx" => {
            Some(("typescript-language-server", vec!["--stdio"]))
        }
        "go" => Some(("gopls", vec![])),
        "c" | "cpp" => Some(("clangd", vec![])),
        "dart" => Some(("dart", vec!["language-server"])),
        "ruby" => Some(("solargraph", vec!["stdio"])),
        "kotlin" => Some(("kotlin-language-server", vec![])),
        "java" => Some(("jdtls", vec!["-data", &jdtls_data])),
        "swift" => Some(("sourcekit-lsp", vec![])),
        "html" => Some(("vscode-html-language-server", vec!["--stdio"])),
        "css" => Some(("vscode-css-language-server", vec!["--stdio"])),
        "vue" => Some(("vue-language-server", vec!["--stdio"])),
        "svelte" => Some(("svelteserver", vec!["--stdio"])),
        _ => None,
    };
    let mut diagnostics = vec![];

    if let Some((cmd, args)) = lsp_cmd {
        let servers_arc = lsp::get_servers();

        // Scope 1: Initialize server and sync file
        let mut start_version = 0;
        let mut client_found = false;

        // Scope 1: Initialize server and sync file
        {
            let mut servers = servers_arc.lock().unwrap();
            if !servers.contains_key(&lang) {
                let root_uri = format!("file://{}", project_dir);
                let final_cmd = crate::utils::resolve_lsp_executable(&lang, cmd);

                println!(
                    "🚀 Starting LSP server for {}: {} (root: {})",
                    lang, final_cmd, root_uri
                );
                match lsp::LspClient::new(&final_cmd, &args, Some(&root_uri)) {
                    Ok(client) => {
                        servers.insert(lang.clone(), client);
                    }
                    Err(e) => {
                        println!("❌ Failed to start LSP server for {}: {}", lang, e);
                    }
                }
            }

            if let Some(client) = servers.get_mut(&lang) {
                // Write the current code to disk
                if let Some(ref rel_path) = payload.file_path {
                    let dest_path = format!("{}/{}", project_dir, rel_path);
                    if let Some(parent) = std::path::Path::new(&dest_path).parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let _ = fs::write(&dest_path, &payload.code);
                }

                // Get starting version
                start_version = client.get_diagnostics_version(&file_uri);
                client_found = true;

                // Trigger document synchronization/change notification in LSP so that it starts compiling and publishes diagnostics
                let _ = client.notify_file_changed(&file_uri, &payload.code, &lang);
                let _ = client.notify_file_saved(&file_uri);
            }
        } // lock dropped here!

        // Wait dynamically for compiler diagnostics to be published
        if client_found {
            let start_time = std::time::Instant::now();
            let mut updated = false;
            while start_time.elapsed() < std::time::Duration::from_millis(3000) {
                {
                    let servers = servers_arc.lock().unwrap();
                    if let Some(client) = servers.get(&lang) {
                        if client.get_diagnostics_version(&file_uri) > start_version {
                            updated = true;
                            break;
                        }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            if !updated {
                println!("⚠️ Timeout waiting for diagnostics update for {}", lang);
            }
        }

        // Scope 2: Retrieve current diagnostics
        {
            let mut servers = servers_arc.lock().unwrap();
            if let Some(client) = servers.get_mut(&lang) {
                diagnostics = client.get_all_diagnostics(&project_dir);
            }
        }
    }

    Json(DiagnosticsResponse { diagnostics })
}
