use crate::api;
use crate::store;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::pages::editor::utils::FileEntry;

fn detect_language_from_tree(file_tree: &[FileEntry], active_file: Option<&str>) -> String {
    // 1. Check for configuration files first
    if file_tree.iter().any(|e| e.name == "package.json") {
        return "javascript".to_string();
    }
    if file_tree.iter().any(|e| e.name == "index.html") {
        return "javascript".to_string();
    }
    if file_tree.iter().any(|e| e.name == "Cargo.toml") {
        return "rust".to_string();
    }
    if file_tree.iter().any(|e| e.name == "go.mod") {
        return "go".to_string();
    }
    if file_tree.iter().any(|e| e.name == "pubspec.yaml") {
        return "dart".to_string();
    }

    // 2. Fallback to active file extension
    if let Some(file_name) = active_file {
        if let Some(ext) = file_name.split('.').last() {
            match ext.to_lowercase().as_str() {
                "rs" => return "rust".to_string(),
                "go" => return "go".to_string(),
                "py" => return "python".to_string(),
                "dart" => return "dart".to_string(),
                "c" => return "c".to_string(),
                "cpp" | "cc" | "cxx" => return "cpp".to_string(),
                "java" => return "java".to_string(),
                "kt" | "kts" => return "kotlin".to_string(),
                "swift" => return "swift".to_string(),
                "rb" => return "ruby".to_string(),
                "cs" => return "csharp".to_string(),
                "scala" => return "scala".to_string(),
                "pl" | "pm" => return "perl".to_string(),
                "hs" | "lhs" => return "haskell".to_string(),
                "pas" => return "pascal".to_string(),
                "r" | "R" => return "r".to_string(),
                "js" | "jsx" => return "javascript".to_string(),
                "ts" | "tsx" => return "typescript".to_string(),
                _ => {}
            }
        }
    }

    // 3. Fallback to first file extension in tree
    for entry in file_tree {
        if !entry.is_dir {
            if let Some(ext) = entry.name.split('.').last() {
                match ext.to_lowercase().as_str() {
                    "rs" => return "rust".to_string(),
                    "go" => return "go".to_string(),
                    "py" => return "python".to_string(),
                    "dart" => return "dart".to_string(),
                    "c" => return "c".to_string(),
                    "cpp" | "cc" | "cxx" => return "cpp".to_string(),
                    "java" => return "java".to_string(),
                    "kt" | "kts" => return "kotlin".to_string(),
                    "swift" => return "swift".to_string(),
                    "rb" => return "ruby".to_string(),
                    "cs" => return "csharp".to_string(),
                    "scala" => return "scala".to_string(),
                    "pl" | "pm" => return "perl".to_string(),
                    "hs" | "lhs" => return "haskell".to_string(),
                    "pas" => return "pascal".to_string(),
                    "r" | "R" => return "r".to_string(),
                    "js" | "jsx" => return "javascript".to_string(),
                    "ts" | "tsx" => return "typescript".to_string(),
                    _ => {}
                }
            }
        }
    }

    "javascript".to_string()
}

fn should_run_in_terminal(lang: &str, file_tree: &[FileEntry]) -> bool {
    let l = lang.to_lowercase();
    if matches!(
        l.as_str(),
        "rust" | "go" | "python" | "dart" | "c" | "cpp" | "java" | "kotlin" | "swift" | "ruby" | "scala" | "perl" | "haskell" | "pascal" | "r"
    ) {
        return true;
    }
    
    if l == "javascript" || l == "typescript" {
        // Run in terminal only if it's NOT a web project (no package.json and no index.html)
        let has_package_json = file_tree.iter().any(|e| e.name == "package.json");
        let has_index_html = file_tree.iter().any(|e| e.name == "index.html");
        return !has_package_json && !has_index_html;
    }
    
    false
}

fn get_run_command(lang: &str, file_path: &str) -> Option<String> {
    let l = lang.to_lowercase();
    match l.as_str() {
        "rust" => Some("cargo run".to_string()),
        "go" => Some(format!("go run {}", file_path)),
        "python" => Some(format!("python3 {}", file_path)),
        "dart" => Some(format!("dart run {}", file_path)),
        "c" => {
            let basename = file_path.strip_suffix(".c").unwrap_or("main");
            Some(format!("gcc {} -o {} && ./{}", file_path, basename, basename))
        }
        "cpp" => {
            let basename = file_path.strip_suffix(".cpp").unwrap_or("main");
            Some(format!("g++ {} -o {} && ./{}", file_path, basename, basename))
        }
        "java" => Some(format!("java {}", file_path)),
        "kotlin" => {
            let basename = file_path.strip_suffix(".kt").unwrap_or("main");
            Some(format!("kotlinc {} -include-runtime -d {}.jar && java -jar {}.jar", file_path, basename, basename))
        }
        "swift" => Some(format!("swift {}", file_path)),
        "ruby" => Some(format!("ruby {}", file_path)),
        "scala" => Some(format!("scalac {} && scala Main", file_path)),
        "perl" => Some(format!("perl {}", file_path)),
        "haskell" => Some(format!("runhaskell {}", file_path)),
        "pascal" => {
            let basename = file_path.strip_suffix(".pas").unwrap_or("main");
            Some(format!("fpc {} && ./{}", file_path, basename))
        }
        "r" => Some(format!("Rscript {}", file_path)),
        "javascript" => Some(format!("node {}", file_path)),
        "typescript" => Some(format!("npx -y tsx {}", file_path)),
        _ => None,
    }
}

pub fn make_run_code(
    pid: String,
    ppath: String,
    plang: String,
    code: RwSignal<String>,
    is_running: RwSignal<bool>,
    output: RwSignal<String>,
    is_error: RwSignal<bool>,
    current_pid: RwSignal<Option<u32>>,
    preview_url: RwSignal<Option<String>>,
    save_current: Callback<bool>,
    terminal_session_id: RwSignal<Option<String>>,
    bottom_tab: RwSignal<usize>,
    active_tab: Signal<Option<String>>,
    show_snack: Callback<String>,
    file_tree_data: RwSignal<Vec<FileEntry>>,
    terminal_history: RwSignal<Vec<String>>,
    bottom_open: RwSignal<bool>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        if is_running.get_untracked() {
            return;
        }
        save_current.run(true);
        
        let mut lang = plang.clone();
        let path = ppath.clone();
        let pid2 = pid.clone();
        let file_tree = file_tree_data.get_untracked();
        let current_code = code.get_untracked();
        let active_file = active_tab.get_untracked();

        if lang.to_lowercase() == "auto" {
            lang = detect_language_from_tree(&file_tree, active_file.as_deref());
        }
        
        if should_run_in_terminal(&lang, &file_tree) {
            let file_name_opt = active_tab.get_untracked();
            let file_name = match file_name_opt {
                Some(name) => name,
                None => {
                    show_snack.run("No active file open to run.".to_string());
                    return;
                }
            };
            
            let run_cmd = match get_run_command(&lang, &file_name) {
                Some(cmd) => cmd,
                None => {
                    show_snack.run(format!("Unsupported language for terminal execution: {}", lang));
                    return;
                }
            };
            
            is_running.set(true);
            bottom_open.set(true);
            bottom_tab.set(0); // Switch to Terminal tab
            output.set(String::new()); // Clear output for a clean run
            
            let is_initializing = terminal_session_id.get_untracked().is_none();
            if is_initializing {
                terminal_session_id.set(Some("initializing".to_string()));
            }
            
            let terminal_session_id_clone = terminal_session_id.clone();
            let output_clone = output;
            let is_running_clone = is_running;
            let terminal_history_clone = terminal_history.clone();
            let run_cmd_clone = run_cmd.clone();
            let pid_clone = pid2.clone();
            
            spawn_local(async move {
                let mut session_id = terminal_session_id_clone.get_untracked();
                if session_id.is_none() || session_id == Some("initializing".to_string()) {
                    match api::start_terminal_api(&path).await {
                        Ok(new_id) => {
                            terminal_session_id_clone.set(Some(new_id.clone()));
                            session_id = Some(new_id);
                            // Wait a short duration for the shell to spawn and initialize
                            gloo_timers::future::TimeoutFuture::new(500).await;
                        }
                        Err(e) => {
                            terminal_session_id_clone.set(None);
                            output_clone.set(format!("❌ Failed to initialize terminal session: {}\n", e));
                            is_running_clone.set(false);
                            return;
                        }
                    }
                }

                
                if let Some(ref sid) = session_id {
                    let project_name = if let Some(last_slash) = path.rfind('/') {
                        path[last_slash + 1..].to_string()
                    } else {
                        path.clone()
                    };
                    
                    // Display the prompt & command as if typed in the terminal
                    let mut current = output_clone.get_untracked();
                    if !current.is_empty() && !current.ends_with('\n') {
                        current.push('\n');
                    }
                    current.push_str(&format!("{} $ {}\n", project_name, run_cmd_clone));
                    output_clone.set(current);
                    
                    // Add to terminal history
                    let mut hist = terminal_history_clone.get_untracked();
                    if hist.last() != Some(&run_cmd_clone) {
                        hist.push(run_cmd_clone.clone());
                        crate::store::save_terminal_history(&pid_clone, &hist);
                        terminal_history_clone.set(hist);
                    }
                    
                    // Construct command simply and send
                    let full_cmd = format!("{}\n", run_cmd_clone);
                    let _ = api::send_terminal_input_api(sid, &full_cmd).await;
                    is_running_clone.set(false);
                }
            });
        } else {
            // Run in background (Web Projects)
            is_running.set(true);
            bottom_open.set(true);
            output.set("Starting dev server...".to_string());
            is_error.set(false);
            
            let cargo_toml = if lang == "rust" {
                let k = store::file_key(&pid2, "Cargo.toml");
                let v = store::load_file(&k);
                if v.is_empty() { None } else { Some(v) }
            } else {
                None
            };
            
            let preview_url_clone = preview_url.clone();
            let current_pid_clone = current_pid.clone();
            let bottom_tab_clone = bottom_tab;
            
            spawn_local(async move {
                let res = api::run_code(&current_code, &lang, &path, cargo_toml.as_deref()).await;
                match res {
                    Ok(r) => {
                        let mut out = r.output.clone();
                        if !r.error.is_empty() {
                            if !out.is_empty() {
                                out.push('\n');
                            }
                            out.push_str(&r.error);
                        }
                        if out.is_empty() {
                            out = "Dev server started with no output.".to_string();
                        }
                        output.set(out);
                        is_error.set(!r.error.is_empty());
                        current_pid_clone.set(r.pid);
                        if let Some(url) = r.url {
                            preview_url_clone.set(Some(url));
                            bottom_tab_clone.set(1); // Switch to Preview tab
                        }
                    }
                    Err(e) => {
                        output.set(format!("❌ Error: Could not connect to API.\n{e}"));
                        is_error.set(true);
                    }
                }
                is_running.set(false);
            });
        }
    })
}
