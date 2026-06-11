use crate::live_server;
use crate::models::{CodeResponse, RunRequest, StopRequest};
use crate::utils::resolve_project_dir;
use axum::Json;
use regex::Regex;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

static RUNNING_PROCESSES: OnceLock<Arc<Mutex<std::collections::HashMap<u32, Child>>>> =
    OnceLock::new();

fn running_processes() -> &'static Arc<Mutex<std::collections::HashMap<u32, Child>>> {
    RUNNING_PROCESSES.get_or_init(|| Arc::new(Mutex::new(std::collections::HashMap::new())))
}

pub async fn run_code(Json(payload): Json<RunRequest>) -> Json<CodeResponse> {
    let project_dir = resolve_project_dir(&payload.project_path);
    let lang = payload.language.to_lowercase();

    if let Some(response) = try_run_web_project(&project_dir, &payload.project_path).await {
        return Json(response);
    }

    let command = resolve_terminal_command(&project_dir, &lang, payload.file_path.as_deref());
    Json(CodeResponse {
        output: command,
        error: String::new(),
        pid: None,
        url: None,
    })
}

pub async fn stop_process(Json(payload): Json<StopRequest>) -> Json<CodeResponse> {
    let mut output = String::new();
    let mut error = String::new();

    if let Some(pid) = payload.pid {
        let mut procs = running_processes().lock().unwrap();
        if let Some(mut child) = procs.remove(&pid) {
            match child.kill() {
                Ok(_) => output = format!("Stopped process {}.", pid),
                Err(e) => error = format!("Failed to stop process {}: {}", pid, e),
            }
        } else {
            #[cfg(unix)]
            {
                use std::process::Command as SysCommand;
                let result = SysCommand::new("kill")
                    .args(["-TERM", &pid.to_string()])
                    .status();
                match result {
                    Ok(status) if status.success() => {
                        output = format!("Stopped process {}.", pid);
                    }
                    Ok(_) => error = format!("Failed to stop process {}.", pid),
                    Err(e) => error = format!("Failed to stop process {}: {}", pid, e),
                }
            }
            #[cfg(not(unix))]
            {
                error = format!("Process {} not found.", pid);
            }
        }
    }

    if payload.stop_live_server.unwrap_or(false) {
        live_server::stop_live_server_internal();
        if output.is_empty() {
            output = "Stopped live preview server.".to_string();
        }
    }

    if output.is_empty() && error.is_empty() {
        output = "Nothing to stop.".to_string();
    }

    Json(CodeResponse {
        output,
        error,
        pid: None,
        url: None,
    })
}

async fn try_run_web_project(project_dir: &str, project_path: &str) -> Option<CodeResponse> {
    let pkg_path = format!("{}/package.json", project_dir);
    if let Ok(content) = fs::read_to_string(&pkg_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if json
                .get("scripts")
                .and_then(|s| s.get("dev"))
                .and_then(|v| v.as_str())
                .is_some()
            {
                return Some(spawn_dev_server(project_dir));
            }
        }
    }

    let index_path = format!("{}/index.html", project_dir);
    if Path::new(&index_path).is_file() {
        match live_server::ensure_live_server(project_path).await {
            Ok(port) => {
                let url = format!("http://127.0.0.1:{}", port);
                return Some(CodeResponse {
                    output: format!("Live preview started at {}\n", url),
                    error: String::new(),
                    pid: None,
                    url: Some(url),
                });
            }
            Err(e) => {
                return Some(CodeResponse {
                    output: String::new(),
                    error: e,
                    pid: None,
                    url: None,
                });
            }
        }
    }

    None
}

fn spawn_dev_server(project_dir: &str) -> CodeResponse {
    let cmd = detect_dev_command(project_dir);
    let (shell, arg) = if cfg!(windows) {
        ("cmd", "/c")
    } else {
        ("sh", "-c")
    };

    let mut child = match Command::new(shell)
        .arg(arg)
        .arg(&cmd)
        .current_dir(project_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return CodeResponse {
                output: String::new(),
                error: format!("Failed to start dev server: {}", e),
                pid: None,
                url: None,
            };
        }
    };

    let pid = child.id();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let url_slot: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let url_out = url_slot.clone();
    let url_err = url_slot.clone();

    if let Some(out) = stdout {
        thread::spawn(move || {
            let reader = BufReader::new(out);
            for line in reader.lines().map_while(Result::ok) {
                if let Some(url) = extract_url_from_line(&line) {
                    if let Ok(mut guard) = url_out.lock() {
                        if guard.is_none() {
                            *guard = Some(url);
                        }
                    }
                }
            }
        });
    }

    if let Some(err) = stderr {
        thread::spawn(move || {
            let reader = BufReader::new(err);
            for line in reader.lines().map_while(Result::ok) {
                if let Some(url) = extract_url_from_line(&line) {
                    if let Ok(mut guard) = url_err.lock() {
                        if guard.is_none() {
                            *guard = Some(url);
                        }
                    }
                }
            }
        });
    }

    running_processes().lock().unwrap().insert(pid, child);

    let mut detected_url = None;
    for _ in 0..80 {
        thread::sleep(Duration::from_millis(250));
        if let Ok(guard) = url_slot.lock() {
            if guard.is_some() {
                detected_url = guard.clone();
                break;
            }
        }
    }

    let output = format!("Starting dev server with: {}\n", cmd);
    CodeResponse {
        output,
        error: String::new(),
        pid: Some(pid),
        url: detected_url,
    }
}

fn detect_dev_command(project_dir: &str) -> String {
    if Path::new(&format!("{}/pnpm-lock.yaml", project_dir)).exists() {
        return "pnpm run dev".to_string();
    }
    if Path::new(&format!("{}/yarn.lock", project_dir)).exists() {
        return "yarn dev".to_string();
    }
    if Path::new(&format!("{}/bun.lockb", project_dir)).exists()
        || Path::new(&format!("{}/bun.lock", project_dir)).exists()
    {
        return "bun run dev".to_string();
    }
    "npm run dev".to_string()
}

fn extract_url_from_line(line: &str) -> Option<String> {
    static URL_RE: OnceLock<Regex> = OnceLock::new();
    let re = URL_RE.get_or_init(|| {
        Regex::new(r"https?://(?:localhost|127\.0\.0\.1|0\.0\.0\.0):\d+[^\s]*").unwrap()
    });
    re.find(line).map(|m| {
        let url = m.as_str().to_string();
        url.replace("0.0.0.0", "127.0.0.1")
    })
}

fn resolve_terminal_command(project_dir: &str, lang: &str, file_path: Option<&str>) -> String {
    if Path::new(&format!("{}/Cargo.toml", project_dir)).is_file() {
        return "cargo run".to_string();
    }
    if Path::new(&format!("{}/go.mod", project_dir)).is_file() {
        if let Some(fp) = file_path.filter(|p| p.ends_with(".go")) {
            return format!("go run {}", fp);
        }
        if Path::new(&format!("{}/main.go", project_dir)).is_file() {
            return "go run main.go".to_string();
        }
        return "go run .".to_string();
    }
    if Path::new(&format!("{}/pubspec.yaml", project_dir)).is_file() {
        return "dart run".to_string();
    }
    if Path::new(&format!("{}/Package.swift", project_dir)).is_file() {
        return "swift run".to_string();
    }
    if let Ok(entries) = fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(".csproj") {
                return "dotnet run".to_string();
            }
            if name == "pom.xml" {
                return "mvn compile exec:java".to_string();
            }
            if name == "build.gradle" || name == "build.gradle.kts" {
                return "./gradlew run".to_string();
            }
            if name.ends_with(".cabal") {
                let pkg = name.trim_end_matches(".cabal");
                return format!("cabal run {}", pkg);
            }
        }
    }

    if let Some(fp) = file_path {
        match lang {
            "python" if fp.ends_with(".py") => return format!("python3 {}", fp),
            "ruby" if fp.ends_with(".rb") => return format!("ruby {}", fp),
            "c" if fp.ends_with(".c") => {
                let bin = fp.trim_end_matches(".c");
                return format!("gcc {} -o {} && ./{}", fp, bin, bin);
            }
            "cpp" if fp.ends_with(".cpp") || fp.ends_with(".cc") => {
                let bin = fp.split('.').next().unwrap_or("a.out");
                return format!("g++ {} -o {} && ./{}", fp, bin, bin);
            }
            "java" if fp.ends_with(".java") => {
                return format!("javac {} && java {}", fp, java_class_name(fp));
            }
            "kotlin" if fp.ends_with(".kt") => {
                return format!(
                    "kotlinc {} -include-runtime -d app.jar && java -jar app.jar",
                    fp
                )
            }
            "javascript" if fp.ends_with(".js") => return format!("node {}", fp),
            "typescript" if fp.ends_with(".ts") => return format!("npx ts-node {}", fp),
            "dart" if fp.ends_with(".dart") => return format!("dart run {}", fp),
            "scala" if fp.ends_with(".scala") => return format!("scala {}", fp),
            "haskell" if fp.ends_with(".hs") => return format!("runhaskell {}", fp),
            "swift" if fp.ends_with(".swift") => return format!("swift {}", fp),
            _ => {}
        }
    }

    match lang {
        "rust" => "cargo run".to_string(),
        "go" => {
            if Path::new(&format!("{}/main.go", project_dir)).is_file() {
                "go run main.go".to_string()
            } else {
                "go run .".to_string()
            }
        }
        "python" => "python3 main.py".to_string(),
        "ruby" => "ruby main.rb".to_string(),
        "dart" => "dart run".to_string(),
        "java" => "javac *.java && java Main".to_string(),
        "kotlin" => "kotlinc src -include-runtime -d app.jar && java -jar app.jar".to_string(),
        "c" => {
            if Path::new(&format!("{}/main.c", project_dir)).is_file() {
                "gcc main.c -o main && ./main".to_string()
            } else {
                "gcc *.c -o main && ./main".to_string()
            }
        }
        "cpp" => {
            if Path::new(&format!("{}/main.cpp", project_dir)).is_file() {
                "g++ main.cpp -o main && ./main".to_string()
            } else {
                "g++ *.cpp -o main && ./main".to_string()
            }
        }
        "csharp" => "dotnet run".to_string(),
        "swift" => "swift run".to_string(),
        "scala" => "scala run".to_string(),
        "haskell" => "runhaskell main.hs".to_string(),
        "javascript" => {
            if Path::new(&format!("{}/package.json", project_dir)).is_file() {
                "npm start".to_string()
            } else if Path::new(&format!("{}/index.js", project_dir)).is_file() {
                "node index.js".to_string()
            } else {
                "node main.js".to_string()
            }
        }
        "typescript" => {
            if Path::new(&format!("{}/package.json", project_dir)).is_file() {
                "npm run dev".to_string()
            } else if Path::new(&format!("{}/index.ts", project_dir)).is_file() {
                "npx ts-node index.ts".to_string()
            } else {
                "npx ts-node main.ts".to_string()
            }
        }
        other => format!("echo 'No run command configured for {}'", other),
    }
}

fn java_class_name(file_path: &str) -> String {
    file_path
        .rsplit('/')
        .next()
        .unwrap_or("Main")
        .trim_end_matches(".java")
        .to_string()
}
