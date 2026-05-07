use std::fs;

pub fn resolve_project_dir(path: &str) -> String {
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

pub fn find_url_in_output(output: &str) -> Option<String> {
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
pub fn extract_prefix(code: &str, line: u32, character: u32) -> String {
    let lines: Vec<&str> = code.split('\n').collect();
    if let Some(line_str) = lines.get(line as usize) {
        let char_idx = (character as usize).min(line_str.len());
        let before_cursor = &line_str[..char_idx];
        before_cursor.chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '!')
            .collect::<String>()
            .chars()
            .rev()
            .collect()
    } else {
        String::new()
    }
}

pub fn resolve_lsp_executable(lang: &str, cmd: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    
    // 1. Check if it's already in the PATH
    #[cfg(not(windows))]
    {
        if let Ok(output) = std::process::Command::new("which").arg(cmd).output() {
            if output.status.success() {
                return cmd.to_string();
            }
        }
    }
    #[cfg(windows)]
    {
        if let Ok(output) = std::process::Command::new("where").arg(cmd).output() {
            if output.status.success() {
                return cmd.to_string();
            }
        }
    }

    // 2. Check common installation directories
    let mut search_paths = vec![
        format!("/opt/homebrew/bin/{}", cmd),
        format!("/usr/local/bin/{}", cmd),
        format!("{}/.npm-global/bin/{}", home, cmd),
        format!("{}/go/bin/{}", home, cmd),
    ];

    // Language specific paths
    match lang {
        "ruby" => {
            // Check user gems (common on macOS/Linux)
            if let Ok(entries) = std::fs::read_dir(format!("{}/.gem/ruby", home)) {
                for entry in entries.flatten() {
                    let bin_path = entry.path().join("bin").join(cmd);
                    search_paths.push(bin_path.to_string_lossy().to_string());
                }
            }
        }
        "go" => {
            search_paths.push(format!("{}/go/bin/{}", home, cmd));
        }
        "python" => {
            search_paths.push(format!("{}/.local/bin/{}", home, cmd));
        }
        "javascript" | "typescript" => {
            search_paths.push(format!("{}/.npm-global/bin/{}", home, cmd));
            search_paths.push(format!("{}/node_modules/.bin/{}", home, cmd));
        }
        "kotlin" => {
            search_paths.push(format!("/opt/homebrew/bin/{}", cmd));
            search_paths.push(format!("/usr/local/bin/{}", cmd));
        }
        _ => {}
    }

    for path in search_paths {
        if std::path::Path::new(&path).exists() {
            return path;
        }
    }

    cmd.to_string()
}
