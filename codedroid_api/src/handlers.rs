use crate::lsp;
use crate::models::{
    CodeRequest, CodeResponse, CommandRequest, CommandResponse, CompletionRequest,
    CompletionResponse, CopyRequest, CreateDirRequest, DefinitionRequest, DefinitionResponse,
    DeleteRequest, FileInfo, FormatRequest, FormatResponse, HoverRequest, HoverResponse,
    MoveRequest, PackageRequest, PackageResponse, ReadFileRequest, ReadFileResponse,
    ReferencesRequest, ReferencesResponse, ScanProjectRequest, ScanProjectResponse, StopRequest,
    SyncRequest, CreateProjectRequest, CreateProjectResponse,
};
use std::path::Path;
use crate::runner::*;
use crate::utils::resolve_project_dir;
use axum::Json;
use std::fs;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn detect_language_from_files(project_dir: &str) -> String {
    // 1. Direct configuration files / specific project structure indicators
    if fs::metadata(format!("{}/package.json", project_dir)).is_ok() {
        return "javascript".to_string();
    }
    if fs::metadata(format!("{}/index.html", project_dir)).is_ok() {
        return "javascript".to_string();
    }
    if fs::metadata(format!("{}/Cargo.toml", project_dir)).is_ok() {
        return "rust".to_string();
    }
    if fs::metadata(format!("{}/go.mod", project_dir)).is_ok() {
        return "go".to_string();
    }
    if fs::metadata(format!("{}/pubspec.yaml", project_dir)).is_ok() {
        return "dart".to_string();
    }

    // 2. Scan directory recursively for extensions
    let mut ext_counts = std::collections::HashMap::new();
    scan_exts(std::path::Path::new(project_dir), &mut ext_counts, 0);

    // Get the extension with the highest count
    let mut best_ext = None;
    let mut max_count = 0;
    for (ext, count) in ext_counts {
        if count > max_count {
            max_count = count;
            best_ext = Some(ext);
        }
    }

    if let Some(ext) = best_ext {
        match ext.as_str() {
            "rs" => "rust".to_string(),
            "go" => "go".to_string(),
            "py" => "python".to_string(),
            "dart" => "dart".to_string(),
            "c" => "c".to_string(),
            "cpp" | "cc" | "cxx" => "cpp".to_string(),
            "java" => "java".to_string(),
            "kt" | "kts" => "kotlin".to_string(),
            "swift" => "swift".to_string(),
            "rb" => "ruby".to_string(),
            "cs" => "csharp".to_string(),
            "scala" => "scala".to_string(),
            "pl" | "pm" => "perl".to_string(),
            "hs" | "lhs" => "haskell".to_string(),
            "pas" => "pascal".to_string(),
            "r" | "R" => "r".to_string(),
            "js" | "jsx" => "javascript".to_string(),
            "ts" | "tsx" => "typescript".to_string(),
            _ => "javascript".to_string(),
        }
    } else {
        "javascript".to_string()
    }
}

fn scan_exts(dir: &std::path::Path, counts: &mut std::collections::HashMap<String, usize>, depth: usize) {
    if depth > 4 { return; }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name != "node_modules" && name != "target" && name != ".git" && name != "build" && name != "dist" {
                    scan_exts(&path, counts, depth + 1);
                }
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    *counts.entry(ext_lower).or_insert(0) += 1;
                }
            }
        }
    }
}

pub async fn run_code(Json(mut payload): Json<CodeRequest>) -> Json<CodeResponse> {
    let project_dir = resolve_project_dir(&payload.project_path);

    if payload.language.to_lowercase() == "auto" || payload.language.trim().is_empty() {
        payload.language = detect_language_from_files(&project_dir);
        println!("Auto-detected language for execution: {}", payload.language);
    }

    match payload.language.to_lowercase().as_str() {
        "rust" => run_rust(payload, &project_dir),
        "go" => run_go(payload, &project_dir),
        "dart" => run_dart(payload, &project_dir),
        "c" => run_c(payload, &project_dir),
        "cpp" => run_cpp(payload, &project_dir),
        "csharp" => run_csharp(payload, &project_dir),
        "java" => run_java(payload, &project_dir),
        "python" => run_python(payload, &project_dir),
        "kotlin" => run_kotlin(payload, &project_dir),
        "swift" => run_swift(payload, &project_dir),
        "ruby" => run_ruby(payload, &project_dir),
        "r" => run_r(payload, &project_dir),
        "scala" => run_scala(payload, &project_dir),
        "perl" => run_perl(payload, &project_dir),
        "haskell" => run_haskell(payload, &project_dir),
        "pascal" => run_pascal(payload, &project_dir),
        "javascript" | "typescript" => {
            let has_package_json = fs::metadata(format!("{}/package.json", project_dir)).is_ok();
            let has_index_html = fs::metadata(format!("{}/index.html", project_dir)).is_ok();

            println!("Checking project: {}", project_dir);
            println!("  has_package_json: {}", has_package_json);
            println!("  has_index_html: {}", has_index_html);

            if !has_package_json && !has_index_html {
                if let Ok(entries) = fs::read_dir(&project_dir) {
                    println!("  Files in dir:");
                    for entry in entries {
                        if let Ok(e) = entry {
                            println!("    {:?}", e.file_name());
                        }
                    }
                } else {
                    println!("  Could not read dir: {}", project_dir);
                }
            }

            let has_dev_script = if has_package_json {
                let pkg_path = format!("{}/package.json", project_dir);
                if let Ok(content) = fs::read_to_string(pkg_path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        json.get("scripts")
                            .and_then(|s| s.get("dev"))
                            .is_some()
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };

            if has_package_json && has_dev_script {
                run_javascript_framework(payload, &project_dir)
            } else if has_index_html {
                run_vanilla_js(payload, &project_dir)
            } else {
                if payload.language.to_lowercase() == "typescript" {
                    run_typescript(payload, &project_dir)
                } else {
                    run_javascript(payload, &project_dir)
                }
            }
        }
        _ => Json(CodeResponse {
            output: "".to_string(),
            error: format!("Unsupported language: {}", payload.language),
            pid: None,
            url: None,
        }),
    }
}

pub async fn stop_process(Json(payload): Json<StopRequest>) -> Json<CodeResponse> {
    let output = if cfg!(windows) {
        Command::new("taskkill")
            .arg("/F")
            .arg("/T")
            .arg("/PID")
            .arg(payload.pid.to_string())
            .output()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(format!(
                "kill -9 -{} || kill -9 {}",
                payload.pid, payload.pid
            ))
            .output()
    };

    match output {
        Ok(_) => Json(CodeResponse {
            output: "Process stopped successfully.".to_string(),
            error: "".to_string(),
            pid: None,
            url: None,
        }),
        Err(e) => Json(CodeResponse {
            output: "".to_string(),
            error: format!("Failed to stop process: {}", e),
            pid: None,
            url: None,
        }),
    }
}

fn get_dependency_file_info(dir: &str, lang: &str) -> Option<(String, String)> {
    let filename = match lang {
        "rust" => "Cargo.toml".to_string(),
        "go" => "go.mod".to_string(),
        "dart" => "pubspec.yaml".to_string(),
        "python" => "requirements.txt".to_string(),
        "ruby" => "Gemfile".to_string(),
        "scala" => "build.sbt".to_string(),
        "swift" => "Package.swift".to_string(),
        "haskell" => {
            if let Ok(entries) = fs::read_dir(dir) {
                entries
                    .flatten()
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .find(|name| name.ends_with(".cabal"))
                    .unwrap_or_else(|| "project.cabal".to_string())
            } else {
                "project.cabal".to_string()
            }
        }
        "javascript" | "typescript" => "package.json".to_string(),
        "java" | "kotlin" => "pom.xml".to_string(),
        "csharp" => {
            if let Ok(entries) = fs::read_dir(dir) {
                entries
                    .flatten()
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .find(|name| name.ends_with(".csproj"))
                    .unwrap_or_else(|| "Project.csproj".to_string())
            } else {
                "Project.csproj".to_string()
            }
        }
        _ => return None,
    };

    let path = format!("{}/{}", dir, filename);
    if let Ok(content) = fs::read_to_string(&path) {
        Some((filename, content))
    } else {
        None
    }
}

pub async fn add_package(Json(payload): Json<PackageRequest>) -> Json<PackageResponse> {
    let lang = payload.language.to_lowercase();
    let dir = resolve_project_dir(&payload.project_path);

    match lang.as_str() {
        "dart" => {
            let path = format!("{}/pubspec.yaml", dir);
            if !fs::metadata(&path).is_ok() {
                let content = "name: project\ndescription: A new Dart project.\nversion: 1.0.0\nenvironment:\n  sdk: '>=2.17.0 <4.0.0'\ndependencies:\n";
                let _ = fs::write(path, content);
            }
        }
        "go" => {
            let path = format!("{}/go.mod", dir);
            if !fs::metadata(&path).is_ok() {
                let _ = fs::write(path, "module project\n\ngo 1.18\n");
            }
        }
        "rust" => {
            let path = format!("{}/Cargo.toml", dir);
            if !fs::metadata(&path).is_ok() {
                let _ = fs::write(path, "[package]\nname = \"project\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n");
            }
        }
        "csharp" => {
            let path = format!("{}/Project.csproj", dir);
            if !fs::metadata(&path).is_ok() {
                let content = "<Project Sdk=\"Microsoft.NET.Sdk\">\n  <PropertyGroup>\n    <OutputType>Exe</OutputType>\n    <TargetFramework>net7.0</TargetFramework>\n  </PropertyGroup>\n</Project>";
                let _ = fs::write(path, content);
            }
        }
        "python" => {
            let path = format!("{}/requirements.txt", dir);
            if !fs::metadata(&path).is_ok() {
                let _ = fs::write(path, "");
            }
        }
        "ruby" => {
            let path = format!("{}/Gemfile", dir);
            if !fs::metadata(&path).is_ok() {
                let _ = fs::write(path, "source \"https://rubygems.org\"");
            }
        }
        "scala" => {
            let path = format!("{}/build.sbt", dir);
            if !fs::metadata(&path).is_ok() {
                let _ = fs::write(
                    path,
                    "name := \"Project\"\nversion := \"0.1\"\nscalaVersion := \"2.13.12\"",
                );
            }
        }
        "swift" => {
            let path = format!("{}/Package.swift", dir);
            if !fs::metadata(&path).is_ok() {
                let pkg_content = format!("// swift-tools-version: 5.9\nimport PackageDescription\n\nlet package = Package(\n    name: \"Project\",\n    targets: [.executableTarget(name: \"Project\")]\n)");
                let _ = fs::write(path, pkg_content);
            }
        }
        "haskell" => {
            let path = format!("{}/project.cabal", dir);
            if !fs::metadata(&path).is_ok() {
                let cabal_content = "name: project\nversion: 0.1.0.0\nexecutable project\n  main-is: main.hs\n  build-depends: base";
                let _ = fs::write(path, cabal_content);
            }
        }
        "javascript" | "typescript" => {
            let path = format!("{}/package.json", dir);
            if !fs::metadata(&path).is_ok() {
                let pkg_content = r#"{
  "name": "project",
  "version": "1.0.0",
  "main": "main.js",
  "dependencies": {}
}"#;
                let _ = fs::write(path, pkg_content);
            }
        }
        "java" | "kotlin" => {
            let path = format!("{}/pom.xml", dir);
            if !fs::metadata(&path).is_ok() {
                let default_pom = r#"<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.project</groupId>
    <artifactId>project</artifactId>
    <version>1.0-SNAPSHOT</version>
    <dependencies>
    </dependencies>
</project>"#;
                let _ = fs::write(path, default_pom);
            }
        }
        _ => {}
    }

    let (cmd, args) = match lang.as_str() {
        "rust" => (
            "cargo".to_string(),
            vec!["add".to_string(), payload.package.clone()],
        ),
        "go" => (
            "go".to_string(),
            vec!["get".to_string(), payload.package.clone()],
        ),
        "dart" => (
            "dart".to_string(),
            vec![
                "pub".to_string(),
                "add".to_string(),
                payload.package.clone(),
            ],
        ),
        "csharp" => (
            "dotnet".to_string(),
            vec![
                "add".to_string(),
                "package".to_string(),
                payload.package.clone(),
            ],
        ),
        "python" => (
            "pip3".to_string(),
            vec![
                "install".to_string(),
                payload.package.clone(),
                "--break-system-packages".to_string(),
            ],
        ),
        "java" | "kotlin" => (
            "mvn".to_string(),
            vec![
                "dependency:get".to_string(),
                format!("-Dartifact={}", payload.package),
            ],
        ),
        "swift" => (
            "swift".to_string(),
            vec![
                "package".to_string(),
                "add".to_string(),
                payload.package.clone(),
            ],
        ),
        "ruby" => (
            "gem".to_string(),
            vec!["install".to_string(), payload.package.clone()],
        ),
        "r" => (
            "Rscript".to_string(),
            vec![
                "-e".to_string(),
                format!(
                    "install.packages('{}', repos='http://cran.us.r-project.org')",
                    payload.package
                ),
            ],
        ),
        "perl" => (
            "cpan".to_string(),
            vec!["-i".to_string(), payload.package.clone()],
        ),
        "haskell" => (
            "cabal".to_string(),
            vec![
                "install".to_string(),
                "--lib".to_string(),
                payload.package.clone(),
            ],
        ),
        "javascript" | "typescript" => (
            "npm".to_string(),
            vec!["install".to_string(), payload.package.clone()],
        ),
        "scala" => (
            "sh".to_string(),
            vec![
                "-c".to_string(),
                format!(
                    "echo '\nlibraryDependencies += \"{}\"' >> build.sbt",
                    payload.package
                ),
            ],
        ),
        "pascal" => (
            "fppkg".to_string(),
            vec!["install".to_string(), payload.package.clone()],
        ),
        "c" | "cpp" => {
            if std::path::Path::new("/data/data/com.termux").exists() {
                (
                    "pkg".to_string(),
                    vec![
                        "install".to_string(),
                        "-y".to_string(),
                        payload.package.clone(),
                    ],
                )
            } else {
                (
                    "apt-get".to_string(),
                    vec![
                        "install".to_string(),
                        "-y".to_string(),
                        payload.package.clone(),
                    ],
                )
            }
        }
        _ => {
            return Json(PackageResponse {
                output: "".to_string(),
                error: format!("Package management not supported for: {}", lang),
                dependency_file_name: None,
                dependency_file_content: None,
            })
        }
    };

    let run_output = Command::new(cmd).args(args).current_dir(&dir).output();

    let (output_str, error_str, success) = match run_output {
        Ok(out) => (
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
            out.status.success(),
        ),
        Err(e) => (
            String::new(),
            format!("Command execution failed: {}", e),
            false,
        ),
    };

    if success {
        if lang == "python" {
            let req_path = format!("{}/requirements.txt", dir);
            if let Ok(mut content) = fs::read_to_string(&req_path) {
                if !content.lines().any(|l| l.trim() == payload.package.trim()) {
                    if !content.ends_with('\n') && !content.is_empty() {
                        content.push('\n');
                    }
                    content.push_str(&format!("{}\n", payload.package.trim()));
                    let _ = fs::write(&req_path, &content);
                }
            }
        } else if lang == "ruby" {
            let gem_path = format!("{}/Gemfile", dir);
            if let Ok(mut content) = fs::read_to_string(&gem_path) {
                let expected_gem = format!("gem \"{}\"", payload.package.trim());
                if !content.lines().any(|l| {
                    l.trim()
                        .contains(&format!("gem \"{}\"", payload.package.trim()))
                        || l.trim()
                            .contains(&format!("gem '{}'", payload.package.trim()))
                }) {
                    if !content.ends_with('\n') && !content.is_empty() {
                        content.push('\n');
                    }
                    content.push_str(&format!("{}\n", expected_gem));
                    let _ = fs::write(&gem_path, &content);
                }
            }
        } else if lang == "java" || lang == "kotlin" {
            let pom_path = format!("{}/pom.xml", dir);
            if let Ok(content) = fs::read_to_string(&pom_path) {
                let parts: Vec<&str> = payload.package.split(':').collect();
                if parts.len() >= 2 {
                    let group_id = parts[0];
                    let artifact_id = parts[1];
                    let version = if parts.len() >= 3 { parts[2] } else { "latest" };

                    if !content.contains(&format!("<artifactId>{}</artifactId>", artifact_id)) {
                        let dep_xml = format!(
                            "        <dependency>\n            <groupId>{}</groupId>\n            <artifactId>{}</artifactId>\n            <version>{}</version>\n        </dependency>\n",
                            group_id, artifact_id, version
                        );
                        let new_content = if content.contains("<dependencies>") {
                            content
                                .replace("<dependencies>", &format!("<dependencies>\n{}", dep_xml))
                        } else if content.contains("</project>") {
                            content.replace(
                                "</project>",
                                &format!(
                                    "    <dependencies>\n{}\n    </dependencies>\n</project>",
                                    dep_xml
                                ),
                            )
                        } else {
                            content
                        };
                        let _ = fs::write(&pom_path, new_content);
                    }
                }
            }
        }
    }

    let dep_info = get_dependency_file_info(&dir, &lang);
    let (dep_name, dep_content) = if let Some((name, content)) = dep_info {
        (Some(name), Some(content))
    } else {
        (None, None)
    };

    Json(PackageResponse {
        output: output_str,
        error: error_str,
        dependency_file_name: dep_name,
        dependency_file_content: dep_content,
    })
}

pub async fn sync_file(Json(payload): Json<SyncRequest>) -> Json<CodeResponse> {
    let target_path = resolve_project_dir(&payload.path);

    if let Ok(metadata) = fs::metadata(&target_path) {
        if metadata.is_dir() {
            let _ = fs::remove_dir_all(&target_path);
        }
    }

    if let Some(parent) = std::path::Path::new(&target_path).parent() {
        let _ = fs::create_dir_all(parent);
    }

    match fs::write(&target_path, payload.content) {
        Ok(_) => Json(CodeResponse {
            output: "File synced".to_string(),
            error: "".to_string(),
            pid: None,
            url: None,
        }),
        Err(e) => Json(CodeResponse {
            output: "".to_string(),
            error: format!("Failed to sync file: {}", e),
            pid: None,
            url: None,
        }),
    }
}

pub async fn get_completions(Json(payload): Json<CompletionRequest>) -> Json<CompletionResponse> {
    let lang = payload.language.to_lowercase();
    println!(
        "🔍 Completion requested for {}: line {}, char {}",
        lang, payload.line, payload.character
    );

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

    let mut suggestions = vec![];

    if let Some((cmd, args)) = lsp_cmd {
        let servers_arc = lsp::get_servers();
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
            if let Some(ref rel_path) = payload.file_path {
                let dest_path = format!("{}/{}", project_dir, rel_path);
                if let Some(parent) = std::path::Path::new(&dest_path).parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&dest_path, &payload.code);
            }

            match client.get_completions(
                &file_uri,
                &payload.code,
                payload.line,
                payload.character,
                &lang,
            ) {
                Ok(mut sugg) => {
                    suggestions.append(&mut sugg);
                }
                Err(e) => {
                    println!("❌ LSP get_completions failed for {}: {}", lang, e);
                    if e.to_string().contains("Broken pipe") {
                        println!("🔌 Connection lost for {}, removing from cache...", lang);
                        servers.remove(&lang);
                    }
                }
            }
        }
    }

    let prefix = crate::utils::extract_prefix(&payload.code, payload.line, payload.character);

    if suggestions.is_empty() {
        suggestions = lsp::fallback_completions(&payload.code, &prefix);
    }

    suggestions.sort();
    suggestions.dedup_by(|a, b| a.label == b.label);
    suggestions.truncate(50);

    println!("✅ Returning {} suggestions", suggestions.len());
    if !suggestions.is_empty() {
        println!("   Preview: {:?}", &suggestions[..suggestions.len().min(5)]);
    }

    Json(CompletionResponse { suggestions })
}

pub async fn delete_file(Json(payload): Json<DeleteRequest>) -> Json<CodeResponse> {
    let target_path = resolve_project_dir(&payload.path);
    println!(
        "🗑 Deleting file/folder: {} (is_dir: {})",
        target_path, payload.is_dir
    );
    let res = if payload.is_dir {
        fs::remove_dir_all(&target_path)
    } else {
        fs::remove_file(&target_path)
    };
    match res {
        Ok(_) => Json(CodeResponse {
            output: "Deleted successfully".to_string(),
            error: "".to_string(),
            pid: None,
            url: None,
        }),
        Err(e) => Json(CodeResponse {
            output: "".to_string(),
            error: format!("Failed to delete: {}", e),
            pid: None,
            url: None,
        }),
    }
}

pub async fn copy_file(Json(payload): Json<CopyRequest>) -> Json<CodeResponse> {
    let src = resolve_project_dir(&payload.src_path);
    let dest = resolve_project_dir(&payload.dest_path);
    println!(
        "📋 Copying from {} to {} (is_dir: {})",
        src, dest, payload.is_dir
    );

    if let Some(parent) = std::path::Path::new(&dest).parent() {
        let _ = fs::create_dir_all(parent);
    }

    let res = if payload.is_dir {
        copy_dir_all(&src, &dest)
    } else {
        fs::copy(&src, &dest).map(|_| ())
    };

    match res {
        Ok(_) => Json(CodeResponse {
            output: "Copied successfully".to_string(),
            error: "".to_string(),
            pid: None,
            url: None,
        }),
        Err(e) => Json(CodeResponse {
            output: "".to_string(),
            error: format!("Failed to copy: {}", e),
            pid: None,
            url: None,
        }),
    }
}

fn copy_dir_all(
    src: impl AsRef<std::path::Path>,
    dst: impl AsRef<std::path::Path>,
) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub async fn create_dir(Json(payload): Json<CreateDirRequest>) -> Json<CodeResponse> {
    let target_path = resolve_project_dir(&payload.path);
    println!("📁 Creating directory: {}", target_path);
    match fs::create_dir_all(&target_path) {
        Ok(_) => Json(CodeResponse {
            output: "Directory created".to_string(),
            error: "".to_string(),
            pid: None,
            url: None,
        }),
        Err(e) => Json(CodeResponse {
            output: "".to_string(),
            error: format!("Failed to create directory: {}", e),
            pid: None,
            url: None,
        }),
    }
}

pub async fn move_file(Json(payload): Json<MoveRequest>) -> Json<CodeResponse> {
    let src = resolve_project_dir(&payload.src_path);
    let dest = resolve_project_dir(&payload.dest_path);
    println!("🚚 Moving/renaming from {} to {}", src, dest);

    if let Some(parent) = std::path::Path::new(&dest).parent() {
        let _ = fs::create_dir_all(parent);
    }

    match fs::rename(&src, &dest) {
        Ok(_) => Json(CodeResponse {
            output: "Moved successfully".to_string(),
            error: "".to_string(),
            pid: None,
            url: None,
        }),
        Err(e) => Json(CodeResponse {
            output: "".to_string(),
            error: format!("Failed to move: {}", e),
            pid: None,
            url: None,
        }),
    }
}

fn run_formatter(
    lang: &str,
    cmd_name: &str,
    args: Vec<String>,
    temp_file_path: &str,
) -> Result<String, String> {
    let resolved_cmd = crate::utils::resolve_lsp_executable(lang, cmd_name);
    println!("Running formatter: {} {:?}", resolved_cmd, args);

    let output = Command::new(&resolved_cmd).args(&args).output();

    match output {
        Ok(out) => {
            if out.status.success() {
                match fs::read_to_string(temp_file_path) {
                    Ok(formatted) => Ok(formatted),
                    Err(e) => Err(format!("Failed to read formatted file: {}", e)),
                }
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let err_msg = if !stderr.trim().is_empty() {
                    stderr
                } else {
                    stdout
                };
                Err(format!(
                    "Formatter failed (exit code {}): {}",
                    out.status.code().unwrap_or(-1),
                    err_msg
                ))
            }
        }
        Err(e) => {
            let install_hint = match cmd_name {
                "rustfmt" => "Please install rustfmt (e.g. `rustup component add rustfmt` or `pkg install rust` in Termux).",
                "gofmt" => "Please install Go (e.g. `pkg install golang` in Termux).",
                "black" => "Please install black (e.g. `pip install black` or `pkg install black` in Termux).",
                "clang-format" => "Please install clang (e.g. `pkg install clang` in Termux).",
                "prettier" | "npx" => "Please install Node.js/npm (e.g. `pkg install nodejs` in Termux).",
                "dart" => "Please install Dart SDK.",
                "ktlint" => "Please install ktlint.",
                "swiftformat" => "Please install swiftformat.",
                "rufo" => "Please install rufo (Ruby formatter).",
                _ => "Please make sure the formatting tool is installed and in your PATH.",
            };
            Err(format!(
                "Formatter '{}' could not be executed: {}\n\n💡 Hint: {}",
                cmd_name, e, install_hint
            ))
        }
    }
}

pub async fn format_code(Json(payload): Json<FormatRequest>) -> Json<FormatResponse> {
    let project_dir = resolve_project_dir(&payload.project_path);
    let lang = payload.language.to_lowercase();

    let ext = match lang.as_str() {
        "rust" => "rs",
        "go" => "go",
        "python" => "py",
        "dart" => "dart",
        "c" => "c",
        "cpp" | "c++" => "cpp",
        "java" => "java",
        "kotlin" => "kt",
        "swift" => "swift",
        "ruby" => "rb",
        "scala" => "scala",
        "haskell" => "hs",
        "javascript" | "jsx" => "js",
        "typescript" | "tsx" => "ts",
        "html" => "html",
        "css" => "css",
        "vue" => "vue",
        "svelte" => "svelte",
        _ => "txt",
    };

    if ext == "txt" {
        return Json(FormatResponse {
            formatted_code: payload.code,
            error: Some(format!(
                "Formatting not supported for language: {}",
                payload.language
            )),
        });
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_filename = format!(".format_tmp_{}.{}", timestamp, ext);
    let temp_filepath = format!("{}/{}", project_dir, temp_filename);

    let _ = fs::create_dir_all(&project_dir);

    if let Err(e) = fs::write(&temp_filepath, &payload.code) {
        return Json(FormatResponse {
            formatted_code: payload.code,
            error: Some(format!(
                "Failed to write temporary file for formatting: {}",
                e
            )),
        });
    }

    let (cmd, args) = match lang.as_str() {
        "rust" => ("rustfmt", vec![temp_filepath.clone()]),
        "go" => ("gofmt", vec!["-w".to_string(), temp_filepath.clone()]),
        "python" => ("black", vec![temp_filepath.clone()]),
        "dart" => ("dart", vec!["format".to_string(), temp_filepath.clone()]),
        "c" | "cpp" | "c++" | "java" => (
            "clang-format",
            vec!["-i".to_string(), temp_filepath.clone()],
        ),
        "kotlin" => ("ktlint", vec!["-F".to_string(), temp_filepath.clone()]),
        "swift" => ("swiftformat", vec![temp_filepath.clone()]),
        "ruby" => ("rufo", vec![temp_filepath.clone()]),
        "scala" => ("scalafmt", vec![temp_filepath.clone()]),
        "javascript" | "typescript" | "jsx" | "tsx" | "html" | "css" | "vue" | "svelte" => {
            let prettier_cmd = crate::utils::resolve_lsp_executable(&lang, "prettier");
            if prettier_cmd != "prettier" && std::path::Path::new(&prettier_cmd).exists() {
                (
                    "prettier",
                    vec!["--write".to_string(), temp_filepath.clone()],
                )
            } else {
                (
                    "npx",
                    vec![
                        "-y".to_string(),
                        "prettier".to_string(),
                        "--write".to_string(),
                        temp_filepath.clone(),
                    ],
                )
            }
        }
        _ => {
            let _ = fs::remove_file(&temp_filepath);
            return Json(FormatResponse {
                formatted_code: payload.code,
                error: Some(format!(
                    "Formatting not supported for language: {}",
                    payload.language
                )),
            });
        }
    };

    let result = run_formatter(&lang, cmd, args, &temp_filepath);

    let _ = fs::remove_file(&temp_filepath);

    match result {
        Ok(formatted) => Json(FormatResponse {
            formatted_code: formatted,
            error: None,
        }),
        Err(err) => Json(FormatResponse {
            formatted_code: payload.code,
            error: Some(err),
        }),
    }
}

pub async fn get_definition(Json(payload): Json<DefinitionRequest>) -> Json<DefinitionResponse> {
    let lang = payload.language.to_lowercase();
    println!(
        "🔍 Definition requested for {}: line {}, char {}",
        lang, payload.line, payload.character
    );

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

    let mut locations = vec![];

    if let Some((cmd, args)) = lsp_cmd {
        let servers_arc = lsp::get_servers();
        let mut servers = servers_arc.lock().unwrap();
        if !servers.contains_key(&lang) {
            let root_uri = format!("file://{}", project_dir);
            let final_cmd = crate::utils::resolve_lsp_executable(&lang, cmd);

            println!(
                "🚀 Starting LSP server for definition: {} (root: {})",
                final_cmd, root_uri
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
            // Write the current code to disk so LSP picks it up
            if let Some(ref rel_path) = payload.file_path {
                let dest_path = format!("{}/{}", project_dir, rel_path);
                if let Some(parent) = std::path::Path::new(&dest_path).parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&dest_path, &payload.code);
            }

            match client.get_definition(
                &file_uri,
                &payload.code,
                payload.line,
                payload.character,
                &lang,
            ) {
                Ok(locs) => {
                    locations = locs;
                }
                Err(e) => {
                    println!("❌ LSP get_definition failed for {}: {}", lang, e);
                    if e.to_string().contains("Broken pipe") {
                        servers.remove(&lang);
                    }
                }
            }
        }
    }

    Json(DefinitionResponse { locations })
}

pub async fn get_references(Json(payload): Json<ReferencesRequest>) -> Json<ReferencesResponse> {
    let lang = payload.language.to_lowercase();
    println!(
        "🔍 References requested for {}: line {}, char {}",
        lang, payload.line, payload.character
    );

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

    let mut locations = vec![];

    if let Some((cmd, args)) = lsp_cmd {
        let servers_arc = lsp::get_servers();
        let mut servers = servers_arc.lock().unwrap();
        if !servers.contains_key(&lang) {
            let root_uri = format!("file://{}", project_dir);
            let final_cmd = crate::utils::resolve_lsp_executable(&lang, cmd);

            println!(
                "🚀 Starting LSP server for references: {} (root: {})",
                final_cmd, root_uri
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
            // Write the current code to disk so LSP picks it up
            if let Some(ref rel_path) = payload.file_path {
                let dest_path = format!("{}/{}", project_dir, rel_path);
                if let Some(parent) = std::path::Path::new(&dest_path).parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&dest_path, &payload.code);
            }

            match client.get_references(
                &file_uri,
                &payload.code,
                payload.line,
                payload.character,
                &lang,
            ) {
                Ok(locs) => {
                    locations = locs;
                }
                Err(e) => {
                    println!("❌ LSP get_references failed for {}: {}", lang, e);
                    if e.to_string().contains("Broken pipe") {
                        servers.remove(&lang);
                    }
                }
            }
        }
    }

    Json(ReferencesResponse { locations })
}

pub async fn read_file(Json(payload): Json<ReadFileRequest>) -> Json<ReadFileResponse> {
    let target_path = resolve_project_dir(&payload.path);
    match fs::read_to_string(&target_path) {
        Ok(content) => Json(ReadFileResponse {
            content,
            error: "".to_string(),
        }),
        Err(e) => Json(ReadFileResponse {
            content: "".to_string(),
            error: format!("Failed to read file: {}", e),
        }),
    }
}

pub async fn get_hover(Json(payload): Json<HoverRequest>) -> Json<HoverResponse> {
    let lang = payload.language.to_lowercase();
    println!(
        "💡 Hover requested for {}: line {}, char {}",
        lang, payload.line, payload.character
    );

    let project_dir = resolve_project_dir(&payload.project_path);
    let is_absolute = payload.file_path.starts_with('/')
        || payload.file_path.starts_with("Users/")
        || payload.file_path.starts_with("home/")
        || payload.file_path.starts_with("data/");

    let file_uri = if is_absolute {
        let clean_path = if payload.file_path.starts_with('/') {
            payload.file_path.clone()
        } else {
            format!("/{}", payload.file_path)
        };
        format!("file://{}", clean_path)
    } else {
        format!("file://{}/{}", project_dir, payload.file_path)
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
        "java" => Some((
            "jdtls",
            vec![
                "-data",
                &jdtls_data,
                "--jvm-arg=-XX:+UseG1GC",
                "--jvm-arg=-XX:+UseStringDeduplication",
            ],
        )),
        "dart" => Some(("dart", vec!["language-server"])),
        "ruby" => Some(("solargraph", vec!["stdio"])),
        "kotlin" => Some(("kotlin-language-server", vec![])),
        "swift" => Some(("sourcekit-lsp", vec![])),
        "html" => Some(("vscode-html-language-server", vec!["--stdio"])),
        "css" => Some(("vscode-css-language-server", vec!["--stdio"])),
        "vue" => Some(("vtsls", vec!["--stdio"])),
        "svelte" => Some(("svelteserver", vec!["--stdio"])),
        _ => None,
    };

    let mut contents = None;
    let mut error = String::new();

    if let Some((cmd, args)) = lsp_cmd {
        let servers_arc = lsp::get_servers();
        let mut servers = servers_arc.lock().unwrap();

        if !servers.contains_key(&lang) {
            let root_uri = format!("file://{}", project_dir);
            let final_cmd = crate::utils::resolve_lsp_executable(&lang, cmd);

            println!(
                "🚀 Starting LSP server for hover: {} (root: {})",
                final_cmd, root_uri
            );
            match lsp::LspClient::new(&final_cmd, &args, Some(&root_uri)) {
                Ok(client) => {
                    servers.insert(lang.clone(), client);
                }
                Err(e) => {
                    println!("❌ Failed to start LSP server for {}: {}", lang, e);
                    error = e.to_string();
                }
            }
        }

        if let Some(client) = servers.get_mut(&lang) {
            if !is_absolute {
                let dest_path = format!("{}/{}", project_dir, payload.file_path);
                if let Some(parent) = std::path::Path::new(&dest_path).parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&dest_path, &payload.code);
            }

            match client.get_hover(
                &file_uri,
                &payload.code,
                payload.line,
                payload.character,
                &lang,
            ) {
                Ok(res) => {
                    contents = res;
                }
                Err(e) => {
                    println!("❌ LSP get_hover failed for {}: {}", lang, e);
                    error = e.to_string();
                    if e.to_string().contains("Broken pipe") {
                        servers.remove(&lang);
                    }
                }
            }
        }
    } else {
        error = format!("No LSP configured for language: {}", lang);
    }

    Json(HoverResponse { contents, error })
}

pub async fn run_command(Json(payload): Json<CommandRequest>) -> Json<CommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    let (shell, arg) = if cfg!(windows) {
        ("cmd", "/c")
    } else {
        ("sh", "-c")
    };

    let mut cmd = Command::new(shell);
    cmd.arg(arg)
        .arg(&payload.command)
        .current_dir(&dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return Json(CommandResponse {
                output: String::new(),
                error: format!("Failed to spawn command: {}", e),
                success: false,
                pid: None,
            });
        }
    };

    let pid = child.id();
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();

    let stdout_buf = Arc::new(Mutex::new(String::new()));
    let stderr_buf = Arc::new(Mutex::new(String::new()));

    let s_out = stdout_buf.clone();
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        while let Ok(n) = stdout.read(&mut buffer) {
            if n == 0 {
                break;
            }
            let mut buf = s_out.lock().unwrap();
            buf.push_str(&String::from_utf8_lossy(&buffer[..n]));
        }
    });

    let s_err = stderr_buf.clone();
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        while let Ok(n) = stderr.read(&mut buffer) {
            if n == 0 {
                break;
            }
            let mut buf = s_err.lock().unwrap();
            buf.push_str(&String::from_utf8_lossy(&buffer[..n]));
        }
    });

    // Wait up to 3 seconds for quick completion
    let start = std::time::Instant::now();
    let mut status = None;
    while start.elapsed() < Duration::from_secs(3) {
        if let Ok(Some(s)) = child.try_wait() {
            status = Some(s);
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    // Give pipes a short moment to flush
    thread::sleep(Duration::from_millis(100));

    let out = stdout_buf.lock().unwrap().clone();
    let err = stderr_buf.lock().unwrap().clone();

    if status.is_none() {
        // Still running after 3 seconds, return pid so frontend can stop it
        Json(CommandResponse {
            output: out,
            error: err,
            success: true,
            pid: Some(pid),
        })
    } else {
        let success = status.unwrap().success();
        Json(CommandResponse {
            output: out,
            error: err,
            success,
            pid: None,
        })
    }
}

fn scan_dir_recursive(
    base_path: &std::path::Path,
    current_path: &std::path::Path,
    files: &mut Vec<FileInfo>,
) -> std::io::Result<()> {
    if current_path.is_dir() {
        for entry in std::fs::read_dir(current_path)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name == ".git"
                    || name == ".DS_Store"
                    || name == ".dart_tool"
                    || name == ".gradle"
                    || name == ".idea"
                    || name == "node_modules"
                    || name == "build"
                    || name == "target"
                {
                    continue;
                }
            }

            let rel_path = path
                .strip_prefix(base_path)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
                .to_string_lossy()
                .into_owned();

            let is_dir = path.is_dir();
            files.push(FileInfo { rel_path, is_dir });

            if is_dir {
                let _ = scan_dir_recursive(base_path, &path, files);
            }
        }
    }
    Ok(())
}

pub async fn scan_project(Json(payload): Json<ScanProjectRequest>) -> Json<ScanProjectResponse> {
    let project_dir = resolve_project_dir(&payload.project_path);
    let base_path = std::path::Path::new(&project_dir);

    let mut files = Vec::new();
    if !base_path.exists() {
        return Json(ScanProjectResponse {
            files,
            error: "Project directory does not exist".to_string(),
        });
    }

    match scan_dir_recursive(base_path, base_path, &mut files) {
        Ok(_) => Json(ScanProjectResponse {
            files,
            error: "".to_string(),
        }),
        Err(e) => Json(ScanProjectResponse {
            files: Vec::new(),
            error: format!("Failed to scan project directory: {}", e),
        }),
    }
}

pub async fn pick_directory() -> Json<crate::models::PickDirectoryResponse> {
    let res = tokio::task::spawn_blocking(|| {
        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("osascript")
                .args(&["-e", "POSIX path of (choose folder with prompt \"Select Project Directory\")"])
                .output();
            match output {
                Ok(out) if out.status.success() => {
                    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !path.is_empty() {
                        return Ok(path);
                    }
                }
                _ => {}
            }
            return Err("Selection cancelled or failed".to_string());
        }

        #[cfg(target_os = "windows")]
        {
            let output = std::process::Command::new("powershell")
                .args(&[
                    "-Command",
                    "Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.FolderBrowserDialog; if ($f.ShowDialog() -eq 'OK') { $f.SelectedPath }"
                ])
                .output();
            match output {
                Ok(out) if out.status.success() => {
                    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !path.is_empty() {
                        return Ok(path);
                    }
                }
                _ => {}
            }
            return Err("Selection cancelled or failed".to_string());
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            if let Ok(output) = std::process::Command::new("zenity")
                .args(&["--file-selection", "--directory", "--title=Select Project Directory"])
                .output()
            {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        return Ok(path);
                    }
                }
            }
            if let Ok(output) = std::process::Command::new("kdialog")
                .args(&["--getexistingdirectory"])
                .output()
            {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        return Ok(path);
                    }
                }
            }
            Err("No system dialog tool (zenity/kdialog) found or selection cancelled".to_string())
        }
    }).await;

    match res {
        Ok(Ok(path)) => Json(crate::models::PickDirectoryResponse {
            success: true,
            path: Some(path),
            error: None,
        }),
        Ok(Err(e)) => Json(crate::models::PickDirectoryResponse {
            success: false,
            path: None,
            error: Some(e),
        }),
        Err(e) => Json(crate::models::PickDirectoryResponse {
            success: false,
            path: None,
            error: Some(e.to_string()),
        }),
    }
}

fn run_command_in_dir(cmd: &str, args: &[&str], dir: &str) -> bool {
    match std::process::Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .output()
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

pub async fn create_project(
    Json(payload): Json<CreateProjectRequest>,
) -> Json<CreateProjectResponse> {
    let project_dir = resolve_project_dir(&payload.path);
    let project_path = Path::new(&project_dir);
    let parent_dir = project_path.parent().unwrap_or(project_path);
    let parent_dir_str = parent_dir.to_string_lossy().into_owned();

    let _ = fs::create_dir_all(parent_dir);

    let lang = payload.language.to_lowercase();
    let fw = payload.framework.to_lowercase();
    let name = &payload.name;

    let mut created_with_cmd = false;

    match lang.as_str() {
        "rust" => {
            let _ = fs::create_dir_all(&project_dir);
            created_with_cmd = run_command_in_dir("cargo", &["init", "--bin"], &project_dir);
            if !created_with_cmd {
                let _ = fs::create_dir_all(project_path.join("src"));
                let _ = fs::write(
                    project_path.join("Cargo.toml"),
                    format!(
                        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
                        name
                    ),
                );
                let _ = fs::write(
                    project_path.join("src/main.rs"),
                    "fn main() {\n    println!(\"Hello, Rust!\");\n}\n",
                );
            }
        }
        "go" => {
            let _ = fs::create_dir_all(&project_dir);
            created_with_cmd = run_command_in_dir("go", &["mod", "init", name], &project_dir);
            let _ = fs::write(
                project_path.join("main.go"),
                "package main\n\nimport \"fmt\"\n\nfunc main() {\n    fmt.Println(\"Hello, Go!\")\n}\n",
            );
            if !created_with_cmd {
                let _ = fs::write(
                    project_path.join("go.mod"),
                    format!("module {}\n\ngo 1.21\n", name),
                );
            }
        }
        "python" => {
            let _ = fs::create_dir_all(&project_dir);
            let _ = fs::write(
                project_path.join("main.py"),
                "print(\"Hello, Python!\")\n",
            );
            let _ = fs::write(project_path.join("requirements.txt"), "");
        }
        "dart" => {
            created_with_cmd = run_command_in_dir(
                "dart",
                &["create", "--template=console-simple", name],
                &parent_dir_str,
            );
            if !created_with_cmd {
                let _ = fs::create_dir_all(&project_dir);
                let _ = fs::write(
                    project_path.join("pubspec.yaml"),
                    format!(
                        "name: {}\ndescription: A new Dart project.\nversion: 1.0.0\nenvironment:\n  sdk: '>=3.0.0 <4.0.0'\ndependencies:\n",
                        name
                    ),
                );
                let _ = fs::write(
                    project_path.join("main.dart"),
                    "void main() {\n  print(\"Hello, Dart!\");\n}\n",
                );
            }
        }
        "java" => {
            let _ = fs::create_dir_all(&project_dir);
            let _ = fs::write(
                project_path.join("Main.java"),
                "public class Main {\n    public static void main(String[] args) {\n        System.out.println(\"Hello, Java!\");\n    }\n}\n",
            );
            let _ = fs::write(
                project_path.join("pom.xml"),
                format!(
                    r#"<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.project</groupId>
    <artifactId>{}</artifactId>
    <version>1.0-SNAPSHOT</version>
    <dependencies>
    </dependencies>
</project>"#,
                    name
                ),
            );
        }
        "c" => {
            let _ = fs::create_dir_all(&project_dir);
            let _ = fs::write(
                project_path.join("main.c"),
                "#include <stdio.h>\n\nint main() {\n    printf(\"Hello, C!\\n\");\n    return 0;\n}\n",
            );
        }
        "cpp" => {
            let _ = fs::create_dir_all(&project_dir);
            let _ = fs::write(
                project_path.join("main.cpp"),
                "#include <iostream>\n\nint main() {\n    std::cout << \"Hello, C++!\" << std::endl;\n    return 0;\n}\n",
            );
        }
        "csharp" => {
            let _ = fs::create_dir_all(&project_dir);
            created_with_cmd = run_command_in_dir("dotnet", &["new", "console"], &project_dir);
            if !created_with_cmd {
                let _ = fs::write(
                    project_path.join("Program.cs"),
                    "Console.WriteLine(\"Hello, C#!\");\n",
                );
                let _ = fs::write(
                    project_path.join(format!("{}.csproj", name)),
                    "<Project Sdk=\"Microsoft.NET.Sdk\">\n  <PropertyGroup>\n    <OutputType>Exe</OutputType>\n    <TargetFramework>net8.0</TargetFramework>\n  </PropertyGroup>\n</Project>\n",
                );
            }
        }
        "kotlin" => {
            let _ = fs::create_dir_all(&project_dir);
            let _ = fs::write(
                project_path.join("main.kt"),
                "fun main() {\n    println(\"Hello, Kotlin!\")\n}\n",
            );
            let _ = fs::write(
                project_path.join("pom.xml"),
                format!(
                    r#"<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.project</groupId>
    <artifactId>{}</artifactId>
    <version>1.0-SNAPSHOT</version>
    <dependencies>
    </dependencies>
</project>"#,
                    name
                ),
            );
        }
        "swift" => {
            let _ = fs::create_dir_all(&project_dir);
            created_with_cmd = run_command_in_dir("swift", &["package", "init", "--type", "executable"], &project_dir);
            if !created_with_cmd {
                let _ = fs::write(
                    project_path.join("main.swift"),
                    "print(\"Hello, Swift!\")\n",
                );
                let _ = fs::write(
                    project_path.join("Package.swift"),
                    format!(
                        "// swift-tools-version: 5.9\nimport PackageDescription\n\nlet package = Package(\n    name: \"{}\",\n    targets: [.executableTarget(name: \"{}\")]\n)\n",
                        name, name
                    ),
                );
            }
        }
        "ruby" => {
            let _ = fs::create_dir_all(&project_dir);
            let _ = fs::write(
                project_path.join("main.rb"),
                "puts \"Hello, Ruby!\"\n",
            );
            let _ = fs::write(
                project_path.join("Gemfile"),
                "source \"https://rubygems.org\"\n",
            );
        }
        "javascript" | "typescript" => {
            let ext = if lang == "typescript" { "ts" } else { "js" };
            match fw.as_str() {
                "none" | "" => {
                    let _ = fs::create_dir_all(&project_dir);
                    created_with_cmd = run_command_in_dir("npm", &["init", "-y"], &project_dir);
                    let _ = fs::write(
                        project_path.join(format!("main.{}", ext)),
                        format!("console.log(\"Hello, {}!\");\n", lang.to_uppercase()),
                    );
                    if !created_with_cmd {
                        let _ = fs::write(
                            project_path.join("package.json"),
                            format!(
                                "{{\n  \"name\": \"{}\",\n  \"version\": \"1.0.0\",\n  \"main\": \"main.{}\",\n  \"dependencies\": {{}}\n}}\n",
                                name, ext
                            ),
                        );
                    }
                }
                "vanilla" | "react" | "vue" | "svelte" => {
                    let template = match fw.as_str() {
                        "vanilla" => if lang == "typescript" { "vanilla-ts" } else { "vanilla" },
                        "react" => if lang == "typescript" { "react-ts" } else { "react" },
                        "vue" => if lang == "typescript" { "vue-ts" } else { "vue" },
                        "svelte" => if lang == "typescript" { "svelte-ts" } else { "svelte" },
                        _ => "vanilla",
                    };
                    created_with_cmd = run_command_in_dir(
                        "npm",
                        &["create", "vite@latest", name, "--", "--template", template],
                        &parent_dir_str,
                    );
                    if !created_with_cmd {
                        let _ = fs::create_dir_all(&project_dir);
                        if fw == "vanilla" {
                            let _ = fs::write(
                                project_path.join("index.html"),
                                format!("<!DOCTYPE html>\n<html>\n<head><title>{}</title><link rel=\"stylesheet\" href=\"style.css\"></head>\n<body>\n  <div id=\"app\"></div>\n  <script type=\"module\" src=\"/main.{}\"></script>\n</body>\n</html>\n", name, ext)
                            );
                            let _ = fs::write(
                                project_path.join(format!("main.{}", ext)),
                                format!("document.getElementById('app').innerHTML = '<h1>Hello {}!</h1>';\n", name)
                            );
                            let _ = fs::write(
                                project_path.join("style.css"),
                                "body { font-family: sans-serif; display:flex; justify-content:center; align-items:center; height:100vh; margin:0; background:#f0f0f0; }\n"
                            );
                            let _ = fs::write(
                                project_path.join("package.json"),
                                format!(
                                    "{{\n  \"name\": \"{}\",\n  \"type\": \"module\",\n  \"scripts\": {{ \"dev\": \"vite --host 0.0.0.0 --port 0\" }},\n  \"devDependencies\": {{ \"vite\": \"latest\" }}\n}}\n",
                                    name
                                )
                            );
                        } else if fw == "react" {
                            let _ = fs::create_dir_all(project_path.join("src"));
                            let _ = fs::write(
                                project_path.join("index.html"),
                                "<!DOCTYPE html>\n<html>\n<head>\n  <meta charset=\"UTF-8\" />\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n  <title>React App</title>\n</head>\n<body>\n  <div id=\"root\"></div>\n  <script type=\"module\" src=\"/src/main.jsx\"></script>\n</body>\n</html>\n"
                            );
                            let _ = fs::write(
                                project_path.join("src/main.jsx"),
                                "import React from 'react';\nimport ReactDOM from 'react-dom/client';\n\nconst App = () => (\n  <div style={{ textAlign: 'center', fontFamily: 'sans-serif', padding: '1em' }}>\n    <h1 style={{ color: '#61dafb' }}>Hello React!</h1>\n    <p>Welcome to your CodeDroid React project.</p>\n  </div>\n);\n\nReactDOM.createRoot(document.getElementById('root')).render(<App />);\n"
                            );
                            let _ = fs::write(
                                project_path.join("vite.config.js"),
                                "import { defineConfig } from 'vite';\nimport react from '@vitejs/plugin-react';\n\nexport default defineConfig({\n  plugins: [react()],\n});\n"
                            );
                            let _ = fs::write(
                                project_path.join("package.json"),
                                format!(
                                    "{{\n  \"name\": \"{}\",\n  \"type\": \"module\",\n  \"scripts\": {{ \"dev\": \"vite --host 0.0.0.0\" }},\n  \"dependencies\": {{ \"react\": \"^18.0.0\", \"react-dom\": \"^18.0.0\" }},\n  \"devDependencies\": {{ \"vite\": \"^5.0.0\", \"@vitejs/plugin-react\": \"^4.0.0\" }}\n}}\n",
                                    name
                                )
                            );
                        } else if fw == "vue" {
                            let _ = fs::create_dir_all(project_path.join("src"));
                            let _ = fs::write(
                                project_path.join("index.html"),
                                "<!DOCTYPE html>\n<html>\n<head>\n  <meta charset=\"UTF-8\" />\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n  <title>Vue App</title>\n</head>\n<body>\n  <div id=\"app\"></div>\n  <script type=\"module\" src=\"/src/main.js\"></script>\n</body>\n</html>\n"
                            );
                            let _ = fs::write(
                                project_path.join("src/App.vue"),
                                "<template>\n  <main>\n    <h1>Hello Vue!</h1>\n    <p>Welcome to your CodeDroid Vue project.</p>\n  </main>\n</template>\n\n<style>\nmain {\n  text-align: center;\n  padding: 1em;\n  font-family: sans-serif;\n}\nh1 {\n  color: #42b983;\n}\n</style>\n"
                            );
                            let _ = fs::write(
                                project_path.join("src/main.js"),
                                "import { createApp } from 'vue';\nimport App from './App.vue';\ncreateApp(App).mount('#app');\n"
                            );
                            let _ = fs::write(
                                project_path.join("vite.config.js"),
                                "import { defineConfig } from 'vite';\nimport vue from '@vitejs/plugin-vue';\n\nexport default defineConfig({\n  plugins: [vue()],\n});\n"
                            );
                            let _ = fs::write(
                                project_path.join("package.json"),
                                format!(
                                    "{{\n  \"name\": \"{}\",\n  \"type\": \"module\",\n  \"scripts\": {{ \"dev\": \"vite --host 0.0.0.0\" }},\n  \"dependencies\": {{ \"vue\": \"^3.4.0\" }},\n  \"devDependencies\": {{ \"vite\": \"^5.0.0\", \"@vitejs/plugin-vue\": \"^5.0.0\" }}\n}}\n",
                                    name
                                )
                            );
                        } else if fw == "svelte" {
                            let _ = fs::create_dir_all(project_path.join("src"));
                            let _ = fs::write(
                                project_path.join("index.html"),
                                "<!DOCTYPE html>\n<html>\n<head>\n  <meta charset=\"UTF-8\" />\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n  <title>Svelte App</title>\n</head>\n<body>\n  <div id=\"app\"></div>\n  <script type=\"module\" src=\"/src/main.js\"></script>\n</body>\n</html>\n"
                            );
                            let _ = fs::write(
                                project_path.join("src/main.js"),
                                "import App from './App.svelte';\n\nconst app = new App({\n  target: document.getElementById('app'),\n});\n\nexport default app;\n"
                            );
                            let _ = fs::write(
                                project_path.join("src/App.svelte"),
                                "<script>\n  let name = 'Svelte';\n</script>\n\n<main>\n  <h1>Hello {name}!</h1>\n  <p>Welcome to your CodeDroid Svelte project.</p>\n</main>\n\n<style>\n  main {\n    text-align: center;\n    padding: 1em;\n    font-family: sans-serif;\n  }\n  h1 {\n    color: #ff3e00;\n    font-size: 2.5rem;\n  }\n</style>\n"
                            );
                            let _ = fs::write(
                                project_path.join("vite.config.js"),
                                "import { defineConfig } from 'vite';\nimport { svelte } from '@sveltejs/vite-plugin-svelte';\n\nexport default defineConfig({\n  plugins: [svelte()],\n});\n"
                            );
                            let _ = fs::write(
                                project_path.join("package.json"),
                                format!(
                                    "{{\n  \"name\": \"{}\",\n  \"type\": \"module\",\n  \"scripts\": {{ \"dev\": \"vite --host 0.0.0.0\" }},\n  \"dependencies\": {{ \"svelte\": \"^4.0.0\" }},\n  \"devDependencies\": {{ \"vite\": \"^5.0.0\", \"@sveltejs/vite-plugin-svelte\": \"^3.0.0\" }}\n}}\n",
                                    name
                                )
                            );
                        }
                    }
                }
                "nextjs" => {
                    let next_lang = if lang == "typescript" { "--ts" } else { "--js" };
                    created_with_cmd = run_command_in_dir(
                        "npx",
                        &[
                            "-y",
                            "create-next-app@latest",
                            name,
                            "--use-npm",
                            next_lang,
                            "--eslint",
                            "--no-src-dir",
                            "--no-tailwind",
                            "--no-app",
                            "--import-alias",
                            "@/*",
                        ],
                        &parent_dir_str,
                    );
                    if !created_with_cmd {
                        let _ = fs::create_dir_all(project_path.join("app"));
                        let _ = fs::write(
                            project_path.join("app/layout.jsx"),
                            "export default function RootLayout({ children }) {\n  return (\n    <html lang=\"en\">\n      <body style={{ margin: 0, fontFamily: 'sans-serif' }}>{children}</body>\n    </html>\n  );\n}\n"
                        );
                        let _ = fs::write(
                            project_path.join("app/page.jsx"),
                            "export default function Home() {\n  return (\n    <div style={{ textAlign: 'center', padding: '2em' }}>\n      <h1 style={{ color: '#0070f3' }}>Hello Next.js!</h1>\n      <p>Welcome to your CodeDroid Next.js project using App Router.</p>\n    </div>\n  );\n}\n"
                        );
                        let _ = fs::write(
                            project_path.join("package.json"),
                            format!(
                                "{{\n  \"name\": \"{}\",\n  \"private\": true,\n  \"scripts\": {{ \"dev\": \"next dev -H 0.0.0.0 -p 3001\" }},\n  \"dependencies\": {{\n    \"next\": \"^14.2.0\",\n    \"react\": \"^18.3.0\",\n    \"react-dom\": \"^18.3.0\"\n  }}\n}}\n",
                                name
                            )
                        );
                    }
                }
                "remix" => {
                    created_with_cmd = run_command_in_dir(
                        "npx",
                        &[
                            "-y",
                            "create-remix@latest",
                            name,
                            "--template",
                            "remix-run/remix/templates/spa",
                        ],
                        &parent_dir_str,
                    );
                    if !created_with_cmd {
                        let _ = fs::create_dir_all(project_path.join("app/routes"));
                        let _ = fs::write(
                            project_path.join("app/root.jsx"),
                            "import { Links, Meta, Outlet, Scripts, ScrollRestoration } from '@remix-run/react';\n\nexport default function App() {\n  return (\n    <html lang=\"en\">\n      <head>\n        <meta charSet=\"utf-8\" />\n        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n        <Meta />\n        <Links />\n      </head>\n      <body>\n        <Outlet />\n        <ScrollRestoration />\n        <Scripts />\n      </body>\n    </html>\n  );\n}\n"
                        );
                        let _ = fs::write(
                            project_path.join("app/routes/_index.jsx"),
                            "export default function Index() {\n  return (\n    <div style={{ textAlign: 'center', fontFamily: 'sans-serif', padding: '1em' }}>\n      <h1 style={{ color: '#319795' }}>Hello Remix!</h1>\n      <p>Welcome to your CodeDroid Remix project.</p>\n    </div>\n  );\n}\n"
                        );
                        let _ = fs::write(
                            project_path.join("vite.config.js"),
                            "import { vitePlugin as remix } from '@remix-run/dev';\nimport { defineConfig } from 'vite';\n\nexport default defineConfig({\n  plugins: [remix()],\n});\n"
                        );
                        let _ = fs::write(
                            project_path.join("package.json"),
                            format!(
                                "{{\n  \"name\": \"{}\",\n  \"private\": true,\n  \"type\": \"module\",\n  \"scripts\": {{ \"dev\": \"vite --host 0.0.0.0\" }},\n  \"dependencies\": {{\n    \"@remix-run/node\": \"^2.9.0\",\n    \"@remix-run/react\": \"^2.9.0\",\n    \"@remix-run/serve\": \"^2.9.0\",\n    \"isbot\": \"^4.1.0\",\n    \"react\": \"^18.2.0\",\n    \"react-dom\": \"^18.2.0\"\n  }},\n  \"devDependencies\": {{\n    \"@remix-run/dev\": \"^2.9.0\",\n    \"vite\": \"^5.1.0\"\n  }}\n}}",
                                name
                            )
                        );
                    }
                }
                _ => {
                    let _ = fs::create_dir_all(&project_dir);
                    let _ = fs::write(
                        project_path.join(format!("main.{}", ext)),
                        format!("console.log('Hello {}!');\n", name),
                    );
                }
            }
        }
        _ => {
            let _ = fs::create_dir_all(&project_dir);
            let _ = fs::write(
                project_path.join("main.txt"),
                format!("Hello, World!\nProject: {}\n", name),
            );
        }
    }

    Json(crate::models::CreateProjectResponse {
        success: true,
        error: "".to_string(),
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rust_formatting() {
        let code = "fn main(){\nprintln!(\"hello\");\n}".to_string();
        let payload = FormatRequest {
            code,
            language: "rust".to_string(),
            project_path: "./test_project".to_string(),
        };
        let response = format_code(axum::Json(payload)).await;
        if response.0.error.is_none() {
            assert!(response
                .0
                .formatted_code
                .contains("fn main() {\n    println!(\"hello\");\n}"));
        } else {
            println!("Formatter warning/error: {:?}", response.0.error);
        }
    }
}
