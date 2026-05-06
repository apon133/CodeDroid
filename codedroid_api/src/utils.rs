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
