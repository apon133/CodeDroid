use std::fs;

fn create_dir_for_path(path_str: &str) -> std::io::Result<()> {
    let path = std::path::Path::new(path_str);
    let dir_to_create = if path.extension().is_some()
        || path
            .file_name()
            .map_or(false, |n| n.to_string_lossy().starts_with('.'))
    {
        path.parent().unwrap_or(path)
    } else {
        path
    };
    fs::create_dir_all(dir_to_create)
}

pub fn resolve_project_dir(path: &str) -> String {
    let (virtual_prefix, sub_dir) = if path.starts_with("/Codedroid_Projects") {
        (Some("/Codedroid_Projects"), "Codedroid_Projects")
    } else if path.starts_with("/Codedroid_Desktop") {
        (Some("/Codedroid_Desktop"), "Desktop")
    } else if path.starts_with("/Codedroid_Documents") {
        (Some("/Codedroid_Documents"), "Documents")
    } else {
        (None, "")
    };

    if let Some(prefix) = virtual_prefix {
        let relative_path = &path[prefix.len()..]; // e.g. "/project_name" or ""

        // Detect if we are running in Termux/Android
        let is_android = std::env::var("ANDROID_DATA").is_ok()
            || std::path::Path::new("/sdcard").exists()
            || std::path::Path::new("/storage/emulated/0").exists();

        if is_android {
            let android_folder = match sub_dir {
                "Desktop" => "Download/codedroid_desktop",
                "Documents" => "Documents/codedroid",
                _ => "Download/codedroid",
            };

            let sdcard_path = format!("/sdcard/{}{}", android_folder, relative_path);
            let emulated_path = format!("/storage/emulated/0/{}{}", android_folder, relative_path);

            if create_dir_for_path(&sdcard_path).is_ok() {
                sdcard_path
            } else if create_dir_for_path(&emulated_path).is_ok() {
                emulated_path
            } else {
                // Try Termux storage shortcut ~/storage/shared
                if let Ok(home) = std::env::var("HOME") {
                    let termux_shared = format!(
                        "{}/storage/shared/{}{}",
                        home, android_folder, relative_path
                    );
                    if create_dir_for_path(&termux_shared).is_ok() {
                        return termux_shared;
                    }
                }

                // Fallback to cache directory if shared storage isn't setup/permitted yet
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                let cache_path = format!("{}/{}{}", home, sub_dir, relative_path);
                let _ = create_dir_for_path(&cache_path);
                cache_path
            }
        } else {
            // Default desktop path
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            let local_path = format!("{}/{}{}", home, sub_dir, relative_path);
            let _ = create_dir_for_path(&local_path);
            local_path
        }
    } else {
        path.to_string()
    }
}

/// Returns the machine's primary local-network IPv4 address (e.g. 192.168.x.x).
/// Falls back to "localhost" if it can't be determined.
#[allow(dead_code)]
pub fn get_local_ip() -> String {
    // Use a UDP socket trick: connect to a public address (no packet sent)
    // and read what local IP the OS chose for routing.
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                let ip = addr.ip().to_string();
                if ip != "0.0.0.0" && !ip.starts_with("127.") {
                    return ip;
                }
            }
        }
    }
    "localhost".to_string()
}

#[allow(dead_code)]
pub fn find_url_in_output(output: &str) -> Option<String> {
    // Look for patterns like http://localhost:5173 or http://127.0.0.1:3000 or http://192.168.1.5:8080
    let re = regex::Regex::new(r"http://([a-zA-Z0-9\.\-]+):(\d+)").unwrap();
    if let Some(caps) = re.captures(output) {
        let host = caps.get(1).map_or("localhost", |m| m.as_str());
        let port = caps.get(2).map_or("3000", |m| m.as_str());

        // Replace localhost / 127.0.0.1 / 0.0.0.0 with the real LAN IP
        // so that URLs work when opened from a mobile phone on the same WiFi.
        let resolved_host = if host == "localhost" || host == "127.0.0.1" || host == "0.0.0.0" {
            get_local_ip()
        } else {
            host.to_string()
        };

        return Some(format!("http://{}:{}", resolved_host, port));
    }
    None
}
pub fn extract_prefix(code: &str, line: u32, character: u32) -> String {
    let lines: Vec<&str> = code.split('\n').collect();
    if let Some(line_str) = lines.get(line as usize) {
        let line_str = line_str.strip_suffix('\r').unwrap_or(line_str);
        let char_idx = (character as usize).min(line_str.len());
        let before_cursor = &line_str[..char_idx];
        before_cursor
            .chars()
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

    // 2. Check common installation directories.
    // Termux on Android uses $PREFIX = /data/data/com.termux/files/usr
    let termux_prefix =
        std::env::var("PREFIX").unwrap_or_else(|_| "/data/data/com.termux/files/usr".to_string());

    let mut search_paths = vec![
        // Termux (Android) — checked first on Android devices
        format!("{}/bin/{}", termux_prefix, cmd),
        // macOS Homebrew (Apple Silicon)
        format!("/opt/homebrew/bin/{}", cmd),
        // macOS Homebrew (Intel) / Linux
        format!("/usr/local/bin/{}", cmd),
        // Linux system bin
        format!("/usr/bin/{}", cmd),
        // npm global installs (cross-platform)
        format!("{}/.npm-global/bin/{}", home, cmd),
        // Go binaries
        format!("{}/go/bin/{}", home, cmd),
        // Alpine PRoot paths
        format!("{}/alpine/usr/local/bin/{}", home, cmd),
        format!("{}/alpine/usr/bin/{}", home, cmd),
        format!("{}/alpine/bin/{}", home, cmd),
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
            // Termux Python packages
            search_paths.push(format!("{}/bin/{}", termux_prefix, cmd));
        }
        "javascript" | "typescript" | "jsx" | "tsx" | "vue" | "svelte" | "angular" => {
            // npm global (cross-platform)
            search_paths.push(format!("{}/.npm-global/bin/{}", home, cmd));
            search_paths.push(format!("{}/node_modules/.bin/{}", home, cmd));
            // Termux npm global location
            search_paths.push(format!("{}/lib/node_modules/.bin/{}", termux_prefix, cmd));
        }
        "kotlin" => {
            search_paths.push("/usr/share/kotlin/kotlinc/bin/kotlin-language-server".to_string());
            search_paths.push("/usr/share/kotlin/bin/kotlin-language-server".to_string());
            search_paths.push(format!("/opt/homebrew/bin/{}", cmd));
            search_paths.push(format!("/usr/local/bin/{}", cmd));
            // Termux
            search_paths.push(format!("{}/bin/{}", termux_prefix, cmd));
        }
        "java" => {
            search_paths.push("/usr/share/java/jdtls/bin/jdtls".to_string());
            search_paths.push("/usr/share/jdtls/bin/jdtls".to_string());
            search_paths.push(format!("/opt/homebrew/bin/{}", cmd));
            search_paths.push(format!("/usr/local/bin/{}", cmd));
            // Termux
            search_paths.push(format!("{}/bin/{}", termux_prefix, cmd));
        }
        "swift" => {
            search_paths.push("/usr/bin/sourcekit-lsp".to_string());
            // Common path on macOS with Xcode
            search_paths.push("/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/sourcekit-lsp".to_string());
        }
        _ => {}
    }

    let mut resolved_path = None;
    for path in &search_paths {
        if std::path::Path::new(path).exists() {
            resolved_path = Some(path.clone());
            break;
        }
    }

    if let Some(path) = resolved_path {
        log_message(&format!("🔍 [LSP Resolution] Resolved '{}' for language '{}' to '{}'", cmd, lang, path));
        path
    } else {
        log_message(&format!(
            "⚠️ [LSP Resolution] Could not find executable '{}' for language '{}' in searched paths: {:?}",
            cmd, lang, search_paths
        ));
        cmd.to_string()
    }
}

/// Dynamically resolve the TypeScript SDK `lib` directory.
/// Searches Termux, npm-global, Homebrew, /usr/local, and /usr/lib in order.
pub fn resolve_typescript_sdk() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let termux_prefix =
        std::env::var("PREFIX").unwrap_or_else(|_| "/data/data/com.termux/files/usr".to_string());

    let candidates = vec![
        // Termux global npm modules
        format!("{}/lib/node_modules/typescript/lib", termux_prefix),
        // npm global (~/.npm-global)
        format!("{}/.npm-global/lib/node_modules/typescript/lib", home),
        // macOS Homebrew (Apple Silicon)
        "/opt/homebrew/lib/node_modules/typescript/lib".to_string(),
        // macOS Homebrew (Intel) / Linux
        "/usr/local/lib/node_modules/typescript/lib".to_string(),
        // Linux system npm
        "/usr/lib/node_modules/typescript/lib".to_string(),
        // Local node_modules fallback
        format!("{}/node_modules/typescript/lib", home),
    ];

    for path in candidates {
        if std::path::Path::new(&path).exists() {
            return path;
        }
    }

    // Last resort: let the LSP server figure it out itself
    "/usr/local/lib/node_modules/typescript/lib".to_string()
}

pub fn setup_env_path() {
    let mut paths = Vec::new();

    let home =
        std::env::var("HOME").unwrap_or_else(|_| "/data/data/com.termux/files/home".to_string());

    paths.push(format!("{}/.cargo/bin", home));
    paths.push(format!("{}/.local/bin", home));
    paths.push(format!("{}/.npm-global/bin", home));
    paths.push(format!("{}/go/bin", home));

    // Alpine Linux PRoot paths (tools installed inside the distro are available here
    // via proot wrappers, or when commands are invoked inside the PRoot shell)
    let alpine_root = format!("{}/alpine", home);
    paths.push(format!("{}/usr/local/bin", alpine_root));
    paths.push(format!("{}/usr/bin", alpine_root));
    paths.push(format!("{}/bin", alpine_root));
    paths.push(format!("{}/usr/local/sbin", alpine_root));
    paths.push(format!("{}/usr/sbin", alpine_root));
    paths.push(format!("{}/sbin", alpine_root));

    paths.push("/opt/homebrew/bin".to_string());
    paths.push("/opt/homebrew/sbin".to_string());
    paths.push("/usr/local/bin".to_string());
    paths.push("/usr/local/sbin".to_string());
    paths.push("/usr/bin".to_string());
    paths.push("/bin".to_string());
    paths.push("/usr/sbin".to_string());
    paths.push("/sbin".to_string());

    if let Ok(prefix) = std::env::var("PREFIX") {
        paths.push(format!("{}/bin", prefix));
    } else {
        paths.push("/data/data/com.termux/files/usr/bin".to_string());
    }

    // Expose Alpine root for other modules
    std::env::set_var("ALPINE_ROOT", &alpine_root);

    let current_path = std::env::var("PATH").unwrap_or_default();
    let split_char = if cfg!(windows) { ';' } else { ':' };

    let mut unique_paths = Vec::new();
    for p in paths {
        let p_trimmed = p.trim();
        if !p_trimmed.is_empty() && !unique_paths.contains(&p_trimmed.to_string()) {
            unique_paths.push(p_trimmed.to_string());
        }
    }

    for p in current_path.split(split_char) {
        let p_trimmed = p.trim();
        if !p_trimmed.is_empty() && !unique_paths.contains(&p_trimmed.to_string()) {
            unique_paths.push(p_trimmed.to_string());
        }
    }

    let new_path = unique_paths.join(&split_char.to_string());
    std::env::set_var("PATH", new_path);

    // Find a writable temp directory from candidates to prevent permission or presence issues
    let mut resolved_tmp = "/tmp".to_string();
    let candidates = vec![
        std::env::var("TMPDIR").unwrap_or_default(),
        std::env::var("TEMP").unwrap_or_default(),
        std::env::var("TMP").unwrap_or_default(),
        "/tmp".to_string(),
        "/var/tmp".to_string(),
        "/data/data/com.termux/files/usr/tmp".to_string(),
        format!("{}/.tmp", home),
        format!("{}/tmp", home),
    ];

    for candidate in candidates {
        if candidate.is_empty() {
            continue;
        }
        let path = std::path::Path::new(&candidate);
        
        // Ensure path directory structure exists
        if !path.exists() {
            let _ = std::fs::create_dir_all(path);
        }
        
        if path.exists() {
            // Test if it is writable
            let test_file = path.join(".codedroid_tmp_test");
            if std::fs::write(&test_file, "test").is_ok() {
                let _ = std::fs::remove_file(test_file);
                resolved_tmp = candidate;
                break;
            }
        }
    }

    std::env::set_var("TMPDIR", &resolved_tmp);
    std::env::set_var("TMP", &resolved_tmp);
    std::env::set_var("TEMP", &resolved_tmp);
}

use std::sync::{OnceLock, Mutex};

pub static LOGS_BUFFER: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

pub fn log_message(msg: &str) {
    println!("{}", msg);
    let buffer = LOGS_BUFFER.get_or_init(|| Mutex::new(Vec::new()));
    if let Ok(mut lock) = buffer.lock() {
        lock.push(msg.to_string());
        if lock.len() > 1000 {
            lock.remove(0);
        }
    }
}

pub fn get_logs() -> Vec<String> {
    let buffer = LOGS_BUFFER.get_or_init(|| Mutex::new(Vec::new()));
    if let Ok(lock) = buffer.lock() {
        lock.clone()
    } else {
        Vec::new()
    }
}

pub fn escape_shell_arg(arg: &str) -> String {
    if arg.is_empty() {
        return "''".to_string();
    }
    format!("'{}'", arg.replace('\'', "'\\''"))
}


