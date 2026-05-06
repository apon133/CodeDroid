use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs;
use tower_http::cors::CorsLayer;

mod lsp;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[derive(Deserialize)]
struct CodeRequest {
    code: String,
    language: String,
    project_path: String,
    cargo_toml: Option<String>,
}

#[derive(Serialize)]
struct CodeResponse {
    output: String,
    error: String,
    pid: Option<u32>,
    url: Option<String>,
}

fn resolve_project_dir(path: &str) -> String {
    if path.starts_with("/Codedroid_Projects") {
        // Map web virtual path to a real local path
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let local_path = format!("{}/.codedroid_web_cache{}", home, path);
        let _ = fs::create_dir_all(&local_path);
        local_path
    } else {
        path.to_string()
    }
}

async fn run_code(Json(payload): Json<CodeRequest>) -> Json<CodeResponse> {
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
                // Single file execution
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


fn run_javascript_framework(_payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    // Ensure node_modules exists
    if !fs::metadata(format!("{}/node_modules", project_dir)).is_ok() {
        println!("Installing dependencies in {}...", project_dir);
        let install_output = Command::new("npm")
            .arg("install")
            .current_dir(project_dir)
            .output();
            
        match install_output {
            Ok(out) => {
                if !out.status.success() {
                    let err = String::from_utf8_lossy(&out.stderr).to_string();
                    println!("npm install failed: {}", err);
                    return Json(CodeResponse {
                        output: "".to_string(),
                        error: format!("Dependency installation failed. Please check your internet connection or package.json.\n\nError: {}", err),
                        pid: None,
                        url: None,
                    });
                } else {
                    println!("npm install completed successfully.");
                }
            },
            Err(e) => {
                println!("Failed to run npm install: {}", e);
                return Json(CodeResponse {
                    output: "".to_string(),
                    error: format!("Failed to run npm install: {}. Ensure npm is installed.", e),
                    pid: None,
                    url: None,
                });
            }
        }
    }

    let mut cmd = Command::new("npm");
    cmd.args(["run", "dev"]).current_dir(project_dir);
    run_with_timeout_web(cmd)
}

fn run_vanilla_js(_payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let mut cmd = Command::new("npx");
    // Use live-server for auto-reload support
    cmd.args(["-y", "live-server", ".", "--port=0", "--no-browser", "--host=0.0.0.0", "--wait=50"]).current_dir(project_dir);
    run_with_timeout_web(cmd)
}

fn run_with_timeout_web(mut cmd: Command) -> Json<CodeResponse> {
    use std::time::Duration;
    use std::thread;
    use std::sync::{Arc, Mutex};
    use std::io::Read;
    use std::process::Stdio;

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    #[cfg(unix)]
    cmd.process_group(0);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return Json(CodeResponse {
            output: "".to_string(),
            error: format!("Failed to spawn process: {}", e),
            pid: None,
            url: None,
        }),
    };

    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();

    let stdout_buf = Arc::new(Mutex::new(String::new()));
    let stderr_buf = Arc::new(Mutex::new(String::new()));

    let s_out = stdout_buf.clone();
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        while let Ok(n) = stdout.read(&mut buffer) {
            if n == 0 { break; }
            let mut buf = s_out.lock().unwrap();
            buf.push_str(&String::from_utf8_lossy(&buffer[..n]));
        }
    });

    let s_err = stderr_buf.clone();
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        while let Ok(n) = stderr.read(&mut buffer) {
            if n == 0 { break; }
            let mut buf = s_err.lock().unwrap();
            buf.push_str(&String::from_utf8_lossy(&buffer[..n]));
        }
    });

    let start = std::time::Instant::now();
    let mut status = None;
    let mut detected_url = None;

    // Wait up to 10 seconds for the server to start and output a URL
    while start.elapsed() < Duration::from_secs(10) {
        if let Ok(Some(s)) = child.try_wait() {
            status = Some(s);
            break;
        }

        let out = stdout_buf.lock().unwrap().clone();
        let err = stderr_buf.lock().unwrap().clone();
        if let Some(url) = find_url_in_output(&out) {
            detected_url = Some(url);
            break;
        }
        if let Some(url) = find_url_in_output(&err) {
            detected_url = Some(url);
            break;
        }
        
        thread::sleep(Duration::from_millis(200));
    }

    let out = stdout_buf.lock().unwrap().clone();
    let err = stderr_buf.lock().unwrap().clone();

    if status.is_none() {
        Json(CodeResponse {
            output: format!("{}\n[Server is running...]", out),
            error: err,
            pid: Some(child.id()),
            url: detected_url,
        })
    } else {
        Json(CodeResponse {
            output: out,
            error: err,
            pid: None,
            url: None,
        })
    }
}

fn find_url_in_output(output: &str) -> Option<String> {
    // Look for patterns like http://localhost:5173 or http://127.0.0.1:3000 or http://192.168.1.5:8080
    let re = regex::Regex::new(r"http://([a-zA-Z0-9\.]+):(\d+)").unwrap();
    if let Some(caps) = re.captures(output) {
        let host = caps.get(1).map_or("localhost", |m| m.as_str());
        let port = caps.get(2).map_or("3000", |m| m.as_str());
        
        // Skip common false positives if necessary, but generally any http://host:port is a good candidate
        if host == "0.0.0.0" {
            return Some(format!("http://localhost:{}", port));
        }
        return Some(format!("http://{}:{}", host, port));
    }
    None
}

fn run_java(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/Main.java", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let compile = Command::new("javac")
        .arg("Main.java")
        .current_dir(project_dir)
        .output();

    match compile {
        Ok(out) if !out.status.success() => {
            return Json(CodeResponse {
                output: "".to_string(),
                error: String::from_utf8_lossy(&out.stderr).to_string(),
                pid: None,
                url: None,
            });
        }
        _ => {}
    }

    let mut cmd = Command::new("java");
    cmd.arg("Main").current_dir(project_dir);
    run_with_timeout(cmd)
}

fn run_python(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.py", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("python3");
    cmd.arg("main.py")
       .env("PYTHONUNBUFFERED", "1")
       .current_dir(project_dir);
    run_with_timeout(cmd)
}

fn run_kotlin(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.kt", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let compile = Command::new("kotlinc")
        .args(["main.kt", "-include-runtime", "-d", "main.jar"])
        .current_dir(project_dir)
        .output();

    match compile {
        Ok(out) if !out.status.success() => {
            return Json(CodeResponse {
                output: "".to_string(),
                error: String::from_utf8_lossy(&out.stderr).to_string(),
                pid: None,
                url: None,
            });
        }
        _ => {}
    }

    let mut cmd = Command::new("java");
    cmd.args(["-jar", "main.jar"]).current_dir(project_dir);
    run_with_timeout(cmd)
}

fn run_swift(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.swift", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("swift");
    cmd.arg("main.swift").current_dir(project_dir);
    run_with_timeout(cmd)
}

fn run_ruby(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.rb", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("ruby");
    cmd.arg("main.rb").current_dir(project_dir);
    run_with_timeout(cmd)
}

fn run_r(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.R", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("Rscript");
    cmd.arg("main.R").current_dir(project_dir);
    run_with_timeout(cmd)
}

fn run_scala(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.scala", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let compile = Command::new("scalac")
        .arg("main.scala")
        .current_dir(project_dir)
        .output();

    match compile {
        Ok(out) if !out.status.success() => {
            return Json(CodeResponse {
                output: "".to_string(),
                error: String::from_utf8_lossy(&out.stderr).to_string(),
                pid: None,
                url: None,
            });
        }
        _ => {}
    }

    let mut cmd = Command::new("scala");
    cmd.arg("Main").current_dir(project_dir);
    run_with_timeout(cmd)
}

fn run_perl(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.pl", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("perl");
    cmd.arg("main.pl").current_dir(project_dir);
    run_with_timeout(cmd)
}

fn run_haskell(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.hs", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("runhaskell");
    cmd.arg("main.hs").current_dir(project_dir);
    run_with_timeout(cmd)
}

fn run_pascal(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.pas", project_dir);
    let _ = fs::write(&file_path, payload.code);
    let exec_path = format!("{}/main", project_dir);
    
    let compile = Command::new("fpc")
        .arg("main.pas")
        .current_dir(project_dir)
        .output();

    match compile {
        Ok(out) if !out.status.success() => {
            return Json(CodeResponse {
                output: "".to_string(),
                error: String::from_utf8_lossy(&out.stderr).to_string(),
                pid: None,
                url: None,
            });
        }
        _ => {}
    }

    let cmd = Command::new(exec_path);
    run_with_timeout(cmd)
}

fn run_javascript(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    // If the code starts with <, it's likely HTML, don't run it with node
    if payload.code.trim().starts_with('<') {
        return Json(CodeResponse {
            output: "".to_string(),
            error: "Cannot run HTML as a JavaScript script. Please ensure package.json or index.html exists for web projects.".to_string(),
            pid: None,
            url: None,
        });
    }

    let file_path = format!("{}/main.js", project_dir);
    // If a directory exists with the same name, remove it
    if let Ok(metadata) = fs::metadata(&file_path) {
        if metadata.is_dir() {
            let _ = fs::remove_dir_all(&file_path);
        }
    }
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("node");
    cmd.arg("main.js").current_dir(project_dir);
    run_with_timeout(cmd)
}

fn run_typescript(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    if payload.code.trim().starts_with('<') {
        return Json(CodeResponse {
            output: "".to_string(),
            error: "Cannot run HTML as a TypeScript script.".to_string(),
            pid: None,
            url: None,
        });
    }

    let file_path = format!("{}/main.ts", project_dir);
    // If a directory exists with the same name, remove it
    if let Ok(metadata) = fs::metadata(&file_path) {
        if metadata.is_dir() {
            let _ = fs::remove_dir_all(&file_path);
        }
    }
    let _ = fs::write(&file_path, payload.code);

    // Ensure node_modules exists (run npm install if missing)
    if !fs::metadata(format!("{}/node_modules", project_dir)).is_ok() {
        let _ = Command::new("npm")
            .arg("install")
            .current_dir(project_dir)
            .output();
    }

    let mut cmd = Command::new("npx");
    cmd.args(["-y", "tsx", "main.ts"]).current_dir(project_dir);
    run_with_timeout(cmd)
}

fn run_rust(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let _ = fs::create_dir_all(format!("{}/src", project_dir));
    let main_rs_path = format!("{}/src/main.rs", project_dir);
    let _ = fs::write(&main_rs_path, payload.code);

    if let Some(cargo_toml) = payload.cargo_toml {
        let _ = fs::write(format!("{}/Cargo.toml", project_dir), cargo_toml);
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("-q").current_dir(project_dir);
    run_with_timeout(cmd)
}

fn run_go(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.go", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("go");
    cmd.arg("run").arg("main.go").current_dir(project_dir);
    run_with_timeout(cmd)
}

fn run_dart(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.dart", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("dart");
    cmd.arg("run").arg("main.dart").current_dir(project_dir);
    run_with_timeout(cmd)
}

fn run_c(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.c", project_dir);
    let exec_path = format!("{}/main", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let compile = Command::new("gcc")
        .arg("main.c")
        .arg("-o")
        .arg("main")
        .current_dir(project_dir)
        .output();

    match compile {
        Ok(out) if !out.status.success() => {
            return Json(CodeResponse {
                output: "".to_string(),
                error: String::from_utf8_lossy(&out.stderr).to_string(),
                pid: None,
                url: None,
            });
        }
        Err(e) => {
            return Json(CodeResponse {
                output: "".to_string(),
                error: format!("Compilation failed: {}", e),
                pid: None,
                url: None,
            });
        }
        _ => {}
    }

    let cmd = Command::new(&exec_path);
    run_with_timeout(cmd)
}

fn run_cpp(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.cpp", project_dir);
    let exec_path = format!("{}/main", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let compile = Command::new("g++")
        .arg("main.cpp")
        .arg("-o")
        .arg("main")
        .current_dir(project_dir)
        .output();

    match compile {
        Ok(out) if !out.status.success() => {
            return Json(CodeResponse {
                output: "".to_string(),
                error: String::from_utf8_lossy(&out.stderr).to_string(),
                pid: None,
                url: None,
            });
        }
        Err(e) => {
            return Json(CodeResponse {
                output: "".to_string(),
                error: format!("Compilation failed: {}", e),
                pid: None,
                url: None,
            });
        }
        _ => {}
    }

    let cmd = Command::new(&exec_path);
    run_with_timeout(cmd)
}

fn run_csharp(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/Program.cs", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("dotnet");
    cmd.arg("run").current_dir(project_dir);
    run_with_timeout(cmd)
}

fn handle_output(output: std::io::Result<std::process::Output>) -> Json<CodeResponse> {
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            
            Json(CodeResponse {
                output: stdout,
                error: stderr,
                pid: None,
                url: None,
            })
        }
        Err(e) => Json(CodeResponse {
            output: "".to_string(),
            error: format!("Execution failed: {}", e),
            pid: None,
            url: None,
        }),
    }
}

// Helper to run command with a timeout and capture partial output (crucial for Flask/Servers)
fn run_with_timeout(mut cmd: Command) -> Json<CodeResponse> {
    use std::time::Duration;
    use std::thread;
    use std::sync::{Arc, Mutex};
    use std::io::Read;
    use std::process::Stdio;

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    #[cfg(unix)]
    cmd.process_group(0); // Start in a new process group

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return Json(CodeResponse {
            output: "".to_string(),
            error: format!("Failed to spawn process: {}", e),
            pid: None,
            url: None,
        }),
    };

    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();

    let stdout_buf = Arc::new(Mutex::new(String::new()));
    let stderr_buf = Arc::new(Mutex::new(String::new()));

    let s_out = stdout_buf.clone();
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        while let Ok(n) = stdout.read(&mut buffer) {
            if n == 0 { break; }
            let mut buf = s_out.lock().unwrap();
            buf.push_str(&String::from_utf8_lossy(&buffer[..n]));
        }
    });

    let s_err = stderr_buf.clone();
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        while let Ok(n) = stderr.read(&mut buffer) {
            if n == 0 { break; }
            let mut buf = s_err.lock().unwrap();
            buf.push_str(&String::from_utf8_lossy(&buffer[..n]));
        }
    });

    let start = std::time::Instant::now();
    let mut status = None;
    while start.elapsed() < Duration::from_secs(10) {
        if let Ok(Some(s)) = child.try_wait() {
            status = Some(s);
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    // Give pipes a moment to flush
    thread::sleep(Duration::from_millis(200));

    let out = stdout_buf.lock().unwrap().clone();
    let err = stderr_buf.lock().unwrap().clone();

    if status.is_none() {
        Json(CodeResponse {
            output: format!("{}\n[Server is running...]", out),
            error: format!("{}\n[Reached 10s timeout - Process active]", err),
            pid: Some(child.id()),
            url: None,
        })
    } else {
        Json(CodeResponse {
            output: out,
            error: err,
            pid: None,
            url: None,
        })
    }
}

#[derive(Deserialize)]
struct StopRequest {
    pid: u32,
}

async fn stop_process(Json(payload): Json<StopRequest>) -> Json<CodeResponse> {
    // On Unix (macOS/Linux), we can use kill. On Windows, taskkill.
    let output = if cfg!(windows) {
        Command::new("taskkill")
            .arg("/F")
            .arg("/T") // Kill child processes too
            .arg("/PID")
            .arg(payload.pid.to_string())
            .output()
    } else {
        // On Unix, try to kill the process group
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

#[derive(Deserialize)]
struct PackageRequest {
    package: String,
    language: String,
    project_path: String,
}

async fn add_package(Json(payload): Json<PackageRequest>) -> Json<CodeResponse> {
    let lang = payload.language.to_lowercase();
    let dir = resolve_project_dir(&payload.project_path);
    
    // Ensure dependency files exist
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

#[derive(Deserialize)]
struct SyncRequest {
    path: String,
    content: String,
}

async fn sync_file(Json(payload): Json<SyncRequest>) -> Json<CodeResponse> {
    let target_path = resolve_project_dir(&payload.path);
    
    // If a directory exists with the same name, remove it
    if let Ok(metadata) = fs::metadata(&target_path) {
        if metadata.is_dir() {
            let _ = fs::remove_dir_all(&target_path);
        }
    }

    // Ensure parent directory exists
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

#[derive(Deserialize)]
struct CompletionRequest {
    code: String,
    language: String,
    project_path: String,
    line: u32,
    character: u32,
}

#[derive(Serialize)]
struct CompletionResponse {
    suggestions: Vec<lsp::CompletionItem>,
}

async fn get_completions(Json(payload): Json<CompletionRequest>) -> Json<CompletionResponse> {
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
        _ => format!("file://{}/main.txt", project_dir),
    };

    let lsp_cmd = match lang.as_str() {
        "rust" => Some(("rust-analyzer", vec![])),
        "python" => Some(("pylsp", vec![])),
        "javascript" | "typescript" => Some(("typescript-language-server", vec!["--stdio"])),
        "go" => Some(("gopls", vec![])),
        "c" | "cpp" => Some(("clangd", vec![])),
        _ => None,
    };

    let mut suggestions = vec![];

    if let Some((cmd, args)) = lsp_cmd {
        let servers_arc = lsp::get_servers();
        let mut servers = servers_arc.lock().unwrap();
        if !servers.contains_key(&lang) {
            let project_dir = resolve_project_dir(&payload.project_path);
            
            // For Rust, rust-analyzer MUST see a Cargo.toml and src/main.rs to work properly
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
            }

            let root_uri = format!("file://{}", project_dir);
            println!("🚀 Starting LSP server for {}: {} (root: {})", lang, cmd, root_uri);
            match lsp::LspClient::new(cmd, &args, Some(&root_uri)) {
                Ok(client) => {
                    servers.insert(lang.clone(), client);
                }
                Err(e) => {
                    println!("❌ Failed to start LSP server for {}: {}", lang, e);
                }
            }
        }
        
        if let Some(client) = servers.get_mut(&lang) {
            // For Rust, write the code to disk as well to ensure LSP sees it in the project context
            if lang == "rust" {
                let project_dir = resolve_project_dir(&payload.project_path);
                let _ = fs::write(format!("{}/src/main.rs", project_dir), &payload.code);
            }
            
            if let Ok(mut sugg) = client.get_completions(&file_uri, &payload.code, payload.line, payload.character, &lang) {
                suggestions.append(&mut sugg);
            }
        }
    }

    if suggestions.is_empty() {
        suggestions = lsp::fallback_completions(&payload.code);
    }

    // Sort and deduplicate
    suggestions.sort();
    suggestions.dedup_by(|a, b| a.label == b.label);
    // Limit to 50 suggestions to avoid huge payloads
    suggestions.truncate(50);

    println!("✅ Returning {} suggestions", suggestions.len());
    if !suggestions.is_empty() {
        println!("   Preview: {:?}", &suggestions[..suggestions.len().min(5)]);
    }

    Json(CompletionResponse { suggestions })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/run", post(run_code))
        .route("/stop", post(stop_process))
        .route("/add_package", post(add_package))
        .route("/sync_file", post(sync_file))
        .route("/complete", post(get_completions))
        .layer(CorsLayer::permissive());

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("🚀 Server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
