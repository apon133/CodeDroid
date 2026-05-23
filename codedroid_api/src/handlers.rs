use axum::Json;
use std::fs;
use std::process::Command;
use crate::models::{
    CodeRequest, CodeResponse, StopRequest, PackageRequest, SyncRequest,
    CompletionRequest, CompletionResponse, DeleteRequest, CopyRequest, CreateDirRequest
};
use crate::utils::resolve_project_dir;
use crate::runner::*;
use crate::lsp;

pub async fn run_code(Json(payload): Json<CodeRequest>) -> Json<CodeResponse> {
    let project_dir = resolve_project_dir(&payload.project_path);

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

            if has_package_json {
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
        },
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
            .arg(format!("kill -9 -{} || kill -9 {}", payload.pid, payload.pid))
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

pub async fn add_package(Json(payload): Json<PackageRequest>) -> Json<CodeResponse> {
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
            if !fs::metadata(&path).is_ok() { let _ = fs::write(path, ""); }
        }
        "ruby" => {
            let path = format!("{}/Gemfile", dir);
            if !fs::metadata(&path).is_ok() { let _ = fs::write(path, "source \"https://rubygems.org\""); }
        }
        "scala" => {
            let path = format!("{}/build.sbt", dir);
            if !fs::metadata(&path).is_ok() { let _ = fs::write(path, "name := \"Project\"\nversion := \"0.1\"\nscalaVersion := \"2.13.12\""); }
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
        _ => {}
    }

    let (cmd, args) = match lang.as_str() {
        "rust" => ("cargo".to_string(), vec!["add".to_string(), payload.package.clone()]),
        "go" => ("go".to_string(), vec!["get".to_string(), payload.package.clone()]),
        "dart" => ("dart".to_string(), vec!["pub".to_string(), "add".to_string(), payload.package.clone()]),
        "csharp" => ("dotnet".to_string(), vec!["add".to_string(), "package".to_string(), payload.package.clone()]),
        "python" => ("pip3".to_string(), vec!["install".to_string(), payload.package.clone(), "--break-system-packages".to_string()]),
        "java" | "kotlin" => ("mvn".to_string(), vec!["dependency:get".to_string(), format!("-Dartifact={}", payload.package)]),
        "swift" => ("swift".to_string(), vec!["package".to_string(), "add".to_string(), payload.package.clone()]),
        "ruby" => ("gem".to_string(), vec!["install".to_string(), payload.package.clone()]),
        "r" => ("Rscript".to_string(), vec!["-e".to_string(), format!("install.packages('{}', repos='http://cran.us.r-project.org')", payload.package)]),
        "perl" => ("cpan".to_string(), vec!["-i".to_string(), payload.package.clone()]),
        "haskell" => ("cabal".to_string(), vec!["install".to_string(), "--lib".to_string(), payload.package.clone()]),
        "javascript" | "typescript" => ("npm".to_string(), vec!["install".to_string(), payload.package.clone()]),
        "scala" => ("sh".to_string(), vec!["-c".to_string(), format!("echo '\nlibraryDependencies += \"{}\"' >> build.sbt", payload.package)]),
        "pascal" => ("fppkg".to_string(), vec!["install".to_string(), payload.package.clone()]),
        "c" | "cpp" => {
            if std::path::Path::new("/data/data/com.termux").exists() {
                ("pkg".to_string(), vec!["install".to_string(), "-y".to_string(), payload.package.clone()])
            } else {
                ("apt-get".to_string(), vec!["install".to_string(), "-y".to_string(), payload.package.clone()])
            }
        },
        _ => return Json(CodeResponse {
            output: "".to_string(),
            error: format!("Package management not supported for: {}", lang),
            pid: None,
            url: None,
        }),
    };

    let output = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .output();

    handle_output(output)
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
    println!("🔍 Completion requested for {}: line {}, char {}", lang, payload.line, payload.character);

    let project_dir = resolve_project_dir(&payload.project_path);
    let file_uri = match lang.as_str() {
        "rust" => format!("file://{}/src/main.rs", project_dir),
        "python" => format!("file://{}/main.py", project_dir),
        "javascript" => format!("file://{}/main.js", project_dir),
        "typescript" => format!("file://{}/main.ts", project_dir),
        "go" => format!("file://{}/main.go", project_dir),
        "c" => format!("file://{}/main.c", project_dir),
        "cpp" => format!("file://{}/main.cpp", project_dir),
        "java" => format!("file://{}/main.java", project_dir),
        "dart" => format!("file://{}/lib/main.dart", project_dir),
        "ruby" => format!("file://{}/main.rb", project_dir),
        "kotlin" => format!("file://{}/main.kt", project_dir),
        "swift" => format!("file://{}/main.swift", project_dir),
        _ => format!("file://{}/main.txt", project_dir),
    };

    let jdtls_data = format!("{}/.jdtls_data", project_dir);
    let lsp_cmd = match lang.as_str() {
        "rust" => Some(("rust-analyzer", vec![])),
        "python" => Some(("pylsp", vec![])),
        "javascript" | "typescript" => Some(("typescript-language-server", vec!["--stdio"])),
        "go" => Some(("gopls", vec![])),
        "c" | "cpp" => Some(("clangd", vec![])),
        "dart" => Some(("dart", vec!["language-server"])),
        "ruby" => Some(("solargraph", vec!["stdio"])),
        "kotlin" => Some(("kotlin-language-server", vec![])),
        "java" => Some(("jdtls", vec!["-data", &jdtls_data])),
        "swift" => Some(("sourcekit-lsp", vec![])),
        _ => None,
    };

    let mut suggestions = vec![];

    if let Some((cmd, args)) = lsp_cmd {
        let servers_arc = lsp::get_servers();
        let mut servers = servers_arc.lock().unwrap();
        if !servers.contains_key(&lang) {
            if lang == "rust" {
                let _ = fs::create_dir_all(format!("{}/src", project_dir));
                let cargo_path = format!("{}/Cargo.toml", project_dir);
                if !std::path::Path::new(&cargo_path).exists() {
                    println!("📝 Creating default Cargo.toml for LSP");
                    let default_cargo = r#"[package]
name = "codedroid_project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
                    let _ = fs::write(cargo_path, default_cargo);
                }
            } else if lang == "go" {
                let _ = fs::create_dir_all(&project_dir);
                let mod_path = format!("{}/go.mod", project_dir);
                if !std::path::Path::new(&mod_path).exists() {
                    println!("📝 Creating default go.mod for LSP");
                    let default_mod = "module codedroid_project\n\ngo 1.25\n";
                    let _ = fs::write(mod_path, default_mod);
                }
            } else if lang == "dart" {
                let _ = fs::create_dir_all(format!("{}/lib", project_dir));
                let pubspec_path = format!("{}/pubspec.yaml", project_dir);
                if !std::path::Path::new(&pubspec_path).exists() {
                    println!("📝 Creating default pubspec.yaml for LSP");
                    let default_pubspec = r#"name: codedroid_project
description: A new Dart project.
version: 1.0.0
environment:
  sdk: '>=3.0.0 <4.0.0'
"#;
                    let _ = fs::write(pubspec_path, default_pubspec);
                }
            }

            let root_uri = format!("file://{}", project_dir);
            let final_cmd = crate::utils::resolve_lsp_executable(&lang, cmd);

            println!("🚀 Starting LSP server for {}: {} (root: {})", lang, final_cmd, root_uri);
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
            match lang.as_str() {
                "rust" => { let _ = fs::write(format!("{}/src/main.rs", project_dir), &payload.code); },
                "dart" => { let _ = fs::write(format!("{}/lib/main.dart", project_dir), &payload.code); },
                "cpp" => { let _ = fs::write(format!("{}/main.cpp", project_dir), &payload.code); },
                "c" => { let _ = fs::write(format!("{}/main.c", project_dir), &payload.code); },
                "python" => { let _ = fs::write(format!("{}/main.py", project_dir), &payload.code); },
                "go" => { let _ = fs::write(format!("{}/main.go", project_dir), &payload.code); },
                "ruby" => { let _ = fs::write(format!("{}/main.rb", project_dir), &payload.code); },
                "javascript" => { let _ = fs::write(format!("{}/main.js", project_dir), &payload.code); },
                "typescript" => { let _ = fs::write(format!("{}/main.ts", project_dir), &payload.code); },
                "kotlin" => { let _ = fs::write(format!("{}/main.kt", project_dir), &payload.code); },
                "java" => { let _ = fs::write(format!("{}/main.java", project_dir), &payload.code); },
                "swift" => { let _ = fs::write(format!("{}/main.swift", project_dir), &payload.code); },
                _ => {}
            }
            
            match client.get_completions(&file_uri, &payload.code, payload.line, payload.character, &lang) {
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
    println!("🗑 Deleting file/folder: {} (is_dir: {})", target_path, payload.is_dir);
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
    println!("📋 Copying from {} to {} (is_dir: {})", src, dest, payload.is_dir);
    
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

fn copy_dir_all(src: impl AsRef<std::path::Path>, dst: impl AsRef<std::path::Path>) -> std::io::Result<()> {
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

