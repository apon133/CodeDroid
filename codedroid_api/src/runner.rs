use std::process::{Command, Stdio};
use std::fs;
use std::io::Read;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use axum::Json;
use crate::models::{CodeRequest, CodeResponse};
use crate::utils::find_url_in_output;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

pub fn handle_output(output: std::io::Result<std::process::Output>) -> Json<CodeResponse> {
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

pub fn run_with_timeout(mut cmd: Command) -> Json<CodeResponse> {
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

pub fn run_with_timeout_web(mut cmd: Command) -> Json<CodeResponse> {
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

fn get_port_from_package_json(project_dir: &str) -> Option<u16> {
    let pkg_path = format!("{}/package.json", project_dir);
    if let Ok(content) = fs::read_to_string(pkg_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(dev_script) = json.get("scripts")
                .and_then(|s| s.get("dev"))
                .and_then(|d| d.as_str())
            {
                let re_port = regex::Regex::new(r"(?:--port|-p)\s+(\d+)").unwrap();
                if let Some(caps) = re_port.captures(dev_script) {
                    if let Some(p_str) = caps.get(1) {
                        if let Ok(port) = p_str.as_str().parse::<u16>() {
                            return Some(port);
                        }
                    }
                }
                if dev_script.contains("vite") {
                    return Some(5173);
                }
                if dev_script.contains("next dev") {
                    if dev_script.contains("-p ") {
                        let re_next_port = regex::Regex::new(r"-p\s+(\d+)").unwrap();
                        if let Some(caps) = re_next_port.captures(dev_script) {
                            if let Some(p_str) = caps.get(1) {
                                if let Ok(port) = p_str.as_str().parse::<u16>() {
                                    return Some(port);
                                }
                            }
                        }
                    }
                    return Some(3000);
                }
                if dev_script.contains("ng serve") {
                    return Some(4200);
                }
            }
        }
    }
    None
}

pub fn run_javascript_framework(_payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    // Ensure node_modules exists
    if !fs::metadata(format!("{}/node_modules", project_dir)).is_ok() {
        println!("Installing dependencies in {}...", project_dir);
        let install_output = Command::new("npm")
            .arg("install")
            .current_dir(project_dir)
            .env("NG_CLI_ANALYTICS", "false")
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

    let fallback_port = get_port_from_package_json(project_dir);

    let mut cmd = Command::new("npm");
    cmd.args(["run", "dev"])
        .current_dir(project_dir)
        .env("NG_CLI_ANALYTICS", "false");
    
    let mut response = run_with_timeout_web(cmd);
    if response.url.is_none() {
        if let Some(port) = fallback_port {
            response.url = Some(format!("http://localhost:{}", port));
        }
    }
    response
}

pub fn run_vanilla_js(_payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let mut cmd = Command::new("npx");
    // Use live-server for auto-reload support
    cmd.args(["-y", "live-server", ".", "--port=0", "--no-browser", "--host=0.0.0.0", "--wait=50"]).current_dir(project_dir);
    run_with_timeout_web(cmd)
}

pub fn run_java(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
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

pub fn run_python(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.py", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("python3");
    cmd.arg("main.py")
       .env("PYTHONUNBUFFERED", "1")
       .current_dir(project_dir);
    run_with_timeout(cmd)
}

pub fn run_kotlin(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
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

pub fn run_swift(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.swift", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("swift");
    cmd.arg("main.swift").current_dir(project_dir);
    run_with_timeout(cmd)
}

pub fn run_ruby(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.rb", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("ruby");
    cmd.arg("main.rb").current_dir(project_dir);
    run_with_timeout(cmd)
}

pub fn run_r(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.R", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("Rscript");
    cmd.arg("main.R").current_dir(project_dir);
    run_with_timeout(cmd)
}

pub fn run_scala(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
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

pub fn run_perl(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.pl", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("perl");
    cmd.arg("main.pl").current_dir(project_dir);
    run_with_timeout(cmd)
}

pub fn run_haskell(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.hs", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("runhaskell");
    cmd.arg("main.hs").current_dir(project_dir);
    run_with_timeout(cmd)
}

pub fn run_pascal(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
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

pub fn run_javascript(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
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

pub fn run_typescript(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
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

pub fn run_rust(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
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

pub fn run_go(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.go", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("go");
    cmd.arg("run").arg("main.go").current_dir(project_dir);
    run_with_timeout(cmd)
}

pub fn run_dart(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/main.dart", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("dart");
    cmd.arg("run").arg("main.dart").current_dir(project_dir);
    run_with_timeout(cmd)
}

pub fn run_c(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
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

pub fn run_cpp(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
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

pub fn run_csharp(payload: CodeRequest, project_dir: &str) -> Json<CodeResponse> {
    let file_path = format!("{}/Program.cs", project_dir);
    let _ = fs::write(&file_path, payload.code);

    let mut cmd = Command::new("dotnet");
    cmd.arg("run").current_dir(project_dir);
    run_with_timeout(cmd)
}
