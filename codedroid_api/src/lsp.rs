use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

static LSP_SERVERS: OnceLock<Arc<Mutex<HashMap<String, LspClient>>>> = OnceLock::new();

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Option<u32>,
    pub code: Option<serde_json::Value>,
    pub source: Option<String>,
    pub message: String,
    #[serde(default)]
    pub file: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct DocumentSymbolResponse {
    pub name: String,
    pub kind: u32,
    pub line: u32,
    pub character: u32,
    #[serde(rename = "containerName")]
    pub container_name: Option<String>,
}

pub fn get_servers() -> Arc<Mutex<HashMap<String, LspClient>>> {
    LSP_SERVERS
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .clone()
}

pub struct LspClient {
    req_id: usize,
    stdin: Arc<Mutex<std::process::ChildStdin>>,
    responses: Arc<Mutex<HashMap<usize, Value>>>,
    opened_files: HashSet<String>,
    file_versions: HashMap<String, i32>,
    diagnostics: Arc<Mutex<HashMap<String, Vec<Diagnostic>>>>,
    diagnostics_version: Arc<Mutex<HashMap<String, usize>>>,
}

impl LspClient {
    pub fn new(cmd: &str, args: &[&str], root_uri: Option<&str>) -> std::io::Result<Self> {
        let mut child_cmd = Command::new(cmd);
        child_cmd
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(uri) = root_uri {
            if uri.starts_with("file://") {
                let mut dir = &uri["file://".len()..];
                while dir.starts_with("//") {
                    dir = &dir[1..];
                }
                let path_exists = std::path::Path::new(dir).exists();
                let log_msg = format!(
                    "root_uri: {}, resolved path: {}, exists: {}\n",
                    uri, dir, path_exists
                );
                let _ = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("/tmp/lsp_spawn.log")
                    .and_then(|mut f| {
                        use std::io::Write;
                        f.write_all(log_msg.as_bytes())
                    });
                if path_exists {
                    child_cmd.current_dir(dir);
                }
            }
        }

        // Set clean environment variables for all LSP servers to ensure stability inside PRoot
        child_cmd.env("HOME", "/root");
        child_cmd.env("TMPDIR", "/tmp");
        child_cmd.env("TMP", "/tmp");
        child_cmd.env("TEMP", "/tmp");
        child_cmd.env(
            "PATH",
            "/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin",
        );
        child_cmd.env("NODE_OPTIONS", "--require /usr/local/lib/node_network_bypass.js");
        child_cmd.env("_JAVA_OPTIONS", "-Djava.net.preferIPv4Stack=true -Djava.net.preferIPv6Addresses=false");
        child_cmd.env("JAVA_TOOL_OPTIONS", "-Djava.net.preferIPv4Stack=true -Djava.net.preferIPv6Addresses=false");

        if cmd == "gopls" || cmd.ends_with("/gopls") {
            child_cmd.env("GOPATH", "/root/go");
            child_cmd.env("GOCACHE", "/tmp/go-cache");
            child_cmd.env("GOTMPDIR", "/tmp");
        }

        let msg = format!("🚀 [LSP Spawn] Spawning server command: '{}' with args: {:?}", cmd, args);
        crate::utils::log_message(&msg);

        let spawn_res = child_cmd.spawn();
        if let Err(ref e) = spawn_res {
            let error_msg = format!("❌ [LSP Spawn Error] Failed to spawn '{}': {}", cmd, e);
            crate::utils::log_message(&error_msg);
        }
        let mut child = spawn_res?;

        let stdin = Arc::new(Mutex::new(child.stdin.take().unwrap()));
        let stdout = child.stdout.take().unwrap();

        let responses = Arc::new(Mutex::new(HashMap::new()));
        let diagnostics = Arc::new(Mutex::new(HashMap::new()));
        let diagnostics_version = Arc::new(Mutex::new(HashMap::new()));
        let stderr = child.stderr.take().unwrap();
        let lang_name = cmd.to_string();
        thread::spawn(move || {
            let mut reader = std::io::BufReader::new(stderr);
            let mut line = String::new();
            while reader.read_line(&mut line).is_ok() {
                if line.is_empty() {
                    break;
                }
                crate::utils::log_message(&format!(" [{} LSP stderr] {}", lang_name, line.trim()));
                line.clear();
            }
        });

        let responses_clone = responses.clone();
        let diagnostics_clone = diagnostics.clone();
        let diagnostics_version_clone = diagnostics_version.clone();
        let stdin_clone = stdin.clone();

        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut line = String::new();
                let mut content_length = 0;
                while let Ok(len) = reader.read_line(&mut line) {
                    if len == 0 {
                        return;
                    } // EOF
                    if line.trim().is_empty() {
                        break;
                    }
                    if line.starts_with("Content-Length:") {
                        let parts: Vec<&str> = line.split(':').collect();
                        if parts.len() == 2 {
                            content_length = parts[1].trim().parse().unwrap_or(0);
                        }
                    }
                    line.clear();
                }

                if content_length > 0 {
                    let mut body = vec![0; content_length];
                    if reader.read_exact(&mut body).is_ok() {
                        if let Ok(val) = serde_json::from_slice::<Value>(&body) {
                            println!("   [LSP Recv] {}", val);
                            if let Some(id) = val.get("id").and_then(|id| id.as_u64()) {
                                responses_clone.lock().unwrap().insert(id as usize, val);
                            } else if let Some(method) = val.get("method").and_then(|m| m.as_str())
                            {
                                if method == "textDocument/publishDiagnostics" {
                                    if let Some(params) = val.get("params") {
                                        if let (Some(uri), Some(diags)) = (
                                            params["uri"].as_str(),
                                            params["diagnostics"].as_array(),
                                        ) {
                                            if let Ok(parsed_diags) =
                                                serde_json::from_value::<Vec<Diagnostic>>(json!(
                                                    diags
                                                ))
                                            {
                                                diagnostics_clone
                                                    .lock()
                                                    .unwrap()
                                                    .insert(uri.to_string(), parsed_diags);
                                                let mut versions =
                                                    diagnostics_version_clone.lock().unwrap();
                                                let entry =
                                                    versions.entry(uri.to_string()).or_insert(0);
                                                *entry += 1;
                                            }
                                        }
                                    }
                                } else if method == "tsserver/request" {
                                    if let Some(params) =
                                        val.get("params").and_then(|p| p.as_array())
                                    {
                                        if let Some(first_param) =
                                            params.first().and_then(|fp| fp.as_array())
                                        {
                                            if let Some(nested_id) =
                                                first_param.first().and_then(|id| id.as_u64())
                                            {
                                                let response = json!({
                                                    "jsonrpc": "2.0",
                                                    "method": "tsserver/response",
                                                    "params": [[nested_id, Value::Null]]
                                                });
                                                let body = response.to_string();
                                                let msg = format!(
                                                    "Content-Length: {}\r\n\r\n{}",
                                                    body.len(),
                                                    body
                                                );
                                                if let Ok(mut stdin_lock) = stdin_clone.lock() {
                                                    let _ = stdin_lock.write_all(msg.as_bytes());
                                                    let _ = stdin_lock.flush();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        let mut client = Self {
            req_id: 1,
            stdin,
            responses,
            opened_files: HashSet::new(),
            file_versions: HashMap::new(),
            diagnostics,
            diagnostics_version,
        };

        let init_options =
            if cmd.contains("typescript") || cmd.contains("vtsls") || cmd.contains("volar") {
                json!({
                    "typescript": {
                        "tsdk": crate::utils::resolve_typescript_sdk()
                    },
                    "vue": {
                        "hybridMode": false
                    }
                })
            } else {
                json!({})
            };

        let init_req = json!({
            "jsonrpc": "2.0",
            "id": client.req_id,
            "method": "initialize",
            "params": {
                "processId": std::process::id(),
                "rootUri": root_uri,
                "initializationOptions": init_options,
                "capabilities": {
                    "textDocument": {
                        "synchronization": {
                            "dynamicRegistration": true,
                            "willSave": true,
                            "willSaveWaitUntil": true,
                            "didSave": true
                        },
                        "completion": {
                            "dynamicRegistration": true,
                            "completionItem": {
                                "snippetSupport": true,
                                "commitCharactersSupport": true,
                                "documentationFormat": ["markdown", "plaintext"],
                                "deprecatedSupport": true,
                                "preselectSupport": true
                            },
                            "contextSupport": true
                        },
                        "hover": { "dynamicRegistration": true },
                        "signatureHelp": { "dynamicRegistration": true },
                        "definition": { "dynamicRegistration": true },
                        "references": { "dynamicRegistration": true },
                        "documentSymbol": { "dynamicRegistration": true, "hierarchicalDocumentSymbolSupport": true }
                    },
                    "workspace": {
                        "workspaceEdit": { "documentChanges": true },
                        "didChangeConfiguration": { "dynamicRegistration": true }
                    }
                }
            }
        });
        client.send_request(&init_req)?;
        let init_id = client.req_id;
        client.req_id += 1;

        // Wait longer for initialization (up to 10s)
        if let Some(resp) = client.wait_for_response_with_timeout(init_id, 100) {
            println!(
                "✅ LSP server initialized: {:?}",
                resp.get("result").and_then(|r| r.get("serverInfo"))
            );

            // Send initialized notification
            let initialized_notif = json!({
                "jsonrpc": "2.0",
                "method": "initialized",
                "params": {}
            });
            client.send_notification(&initialized_notif)?;
        } else {
            println!("⚠️ LSP server failed to initialize within 10s");
        }

        Ok(client)
    }

    fn send_notification(&mut self, notif: &Value) -> std::io::Result<()> {
        let body = notif.to_string();
        println!("   [LSP Send Notif] {}", body);
        let msg = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        let mut stdin = self.stdin.lock().unwrap();
        stdin.write_all(msg.as_bytes())?;
        stdin.flush()?;
        Ok(())
    }

    fn send_request(&mut self, req: &Value) -> std::io::Result<()> {
        let body = req.to_string();
        println!("   [LSP Send Req] {}", body);
        let msg = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        let mut stdin = self.stdin.lock().unwrap();
        stdin.write_all(msg.as_bytes())?;
        stdin.flush()?;
        Ok(())
    }

    fn wait_for_response(&self, id: usize) -> Option<Value> {
        self.wait_for_response_with_timeout(id, 500) // Default 5s (500 * 10ms)
    }

    fn wait_for_response_with_timeout(&self, id: usize, iterations: usize) -> Option<Value> {
        for _ in 0..iterations {
            {
                let mut resps = self.responses.lock().unwrap();
                if let Some(v) = resps.remove(&id) {
                    return Some(v);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        None
    }

    pub fn get_completions(
        &mut self,
        file_uri: &str,
        code: &str,
        line: u32,
        character: u32,
        lang: &str,
    ) -> std::io::Result<Vec<CompletionItem>> {
        self.notify_file_changed(file_uri, code, lang)?;

        // Removed artificial 500ms sleep for performance.
        // LSP servers handle sequential requests correctly.

        let req_id = self.req_id;
        let comp_req = json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": file_uri
                },
                "position": {
                    "line": line,
                    "character": character
                },
                "context": {
                    "triggerKind": 1 // Invoked
                }
            }
        });
        self.send_request(&comp_req)?;
        self.req_id += 1;

        let prefix = crate::utils::extract_prefix(code, line, character);
        println!("   Filtering suggestions with prefix: '{}'", prefix);

        let mut suggestions = Vec::new();
        if let Some(resp) = self.wait_for_response(req_id) {
            let items = resp["result"]["items"]
                .as_array()
                .or_else(|| resp["result"].as_array());

            if let Some(items) = items {
                println!("   LSP result contains {} items", items.len());
                let prefix_low = prefix.to_lowercase();
                for item in items {
                    if let Some(label) = item["label"].as_str() {
                        let label_low = label.to_lowercase();

                        let matches = if prefix.is_empty() {
                            true
                        } else {
                            // Match if label starts with prefix OR if any part (split by ::, ., ->, etc) starts with prefix
                            label_low.starts_with(&prefix_low)
                                || label_low
                                    .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '!')
                                    .any(|part| part.starts_with(&prefix_low))
                        };

                        if matches {
                            let insert_text = item["textEdit"]["newText"]
                                .as_str()
                                .or_else(|| item["insertText"].as_str())
                                .map(|s| s.to_string());

                            suggestions.push(CompletionItem {
                                label: label.to_string(),
                                insert_text,
                                kind: item["kind"].as_u64().map(|k| k as u32),
                                detail: item["detail"].as_str().map(|s| s.to_string()),
                                documentation: item["documentation"]
                                    .as_str()
                                    .or_else(|| item["documentation"]["value"].as_str())
                                    .map(|s| s.to_string()),
                            });
                        }
                    }
                }
            } else {
                println!(
                    "   LSP result is empty or not an array: {:?}",
                    resp["result"]
                );
            }
        } else {
            println!("   ⚠️ LSP timed out for request {}", req_id);
        }

        Ok(suggestions)
    }

    pub fn get_definition(
        &mut self,
        file_uri: &str,
        code: &str,
        line: u32,
        character: u32,
        lang: &str,
    ) -> std::io::Result<Vec<Location>> {
        self.notify_file_changed(file_uri, code, lang)?;

        let req_id = self.req_id;
        let req = json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": file_uri
                },
                "position": {
                    "line": line,
                    "character": character
                }
            }
        });
        self.send_request(&req)?;
        self.req_id += 1;

        let mut locations = Vec::new();
        if let Some(resp) = self.wait_for_response(req_id) {
            let result = &resp["result"];
            if !result.is_null() {
                if let Some(arr) = result.as_array() {
                    for item in arr {
                        if let Some(loc) = parse_location(item) {
                            locations.push(loc);
                        }
                    }
                } else if let Some(loc) = parse_location(result) {
                    locations.push(loc);
                }
            }
        } else {
            println!("   ⚠️ LSP timed out for definition request {}", req_id);
        }
        Ok(locations)
    }

    pub fn get_references(
        &mut self,
        file_uri: &str,
        code: &str,
        line: u32,
        character: u32,
        lang: &str,
    ) -> std::io::Result<Vec<Location>> {
        self.notify_file_changed(file_uri, code, lang)?;

        let req_id = self.req_id;
        let req = json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": file_uri
                },
                "position": {
                    "line": line,
                    "character": character
                },
                "context": {
                    "includeDeclaration": true
                }
            }
        });
        self.send_request(&req)?;
        self.req_id += 1;

        let mut locations = Vec::new();
        if let Some(resp) = self.wait_for_response(req_id) {
            let result = &resp["result"];
            if !result.is_null() {
                if let Some(arr) = result.as_array() {
                    for item in arr {
                        if let Some(loc) = parse_location(item) {
                            locations.push(loc);
                        }
                    }
                } else if let Some(loc) = parse_location(result) {
                    locations.push(loc);
                }
            }
        } else {
            println!("   ⚠️ LSP timed out for references request {}", req_id);
        }
        Ok(locations)
    }

    pub fn get_hover(
        &mut self,
        file_uri: &str,
        code: &str,
        line: u32,
        character: u32,
        lang: &str,
    ) -> std::io::Result<Option<String>> {
        self.notify_file_changed(file_uri, code, lang)?;

        let req_id = self.req_id;
        let req = json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {
                    "uri": file_uri
                },
                "position": {
                    "line": line,
                    "character": character
                }
            }
        });
        self.send_request(&req)?;
        self.req_id += 1;

        if let Some(resp) = self.wait_for_response(req_id) {
            let result = &resp["result"];
            if !result.is_null() {
                let contents = &result["contents"];
                let mut hover_text = String::new();

                fn extract_content_value(val: &Value, out: &mut String) {
                    if let Some(s) = val.as_str() {
                        if !out.is_empty() {
                            out.push_str("\n\n");
                        }
                        out.push_str(s);
                    } else if let Some(obj) = val.as_object() {
                        if let Some(value) = obj.get("value").and_then(|v| v.as_str()) {
                            if !out.is_empty() {
                                out.push_str("\n\n");
                            }
                            if let Some(language) = obj.get("language").and_then(|l| l.as_str()) {
                                out.push_str(&format!("```{}\n{}\n```", language, value));
                            } else {
                                out.push_str(value);
                            }
                        }
                    }
                }

                if let Some(arr) = contents.as_array() {
                    for item in arr {
                        extract_content_value(item, &mut hover_text);
                    }
                } else {
                    extract_content_value(contents, &mut hover_text);
                }

                if !hover_text.is_empty() {
                    return Ok(Some(hover_text));
                }
            }
        }
        Ok(None)
    }

    pub fn get_symbols(
        &mut self,
        file_uri: &str,
        code: &str,
        lang: &str,
    ) -> std::io::Result<Vec<DocumentSymbolResponse>> {
        self.notify_file_changed(file_uri, code, lang)?;

        let req_id = self.req_id;
        let req = json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "method": "textDocument/documentSymbol",
            "params": {
                "textDocument": {
                    "uri": file_uri
                }
            }
        });
        self.send_request(&req)?;
        self.req_id += 1;

        let mut symbols = Vec::new();
        let mut got_response = false;
        if let Some(resp) = self.wait_for_response(req_id) {
            let result = &resp["result"];
            if !result.is_null() {
                got_response = true;
                if let Some(arr) = result.as_array() {
                    for item in arr {
                        parse_symbol_item(item, &mut symbols, None);
                    }
                }
            }
        } else {
            println!("   ⚠️ LSP timed out for documentSymbol request {}", req_id);
        }

        if !got_response || symbols.is_empty() {
            symbols = fallback_symbols(code, lang);
        }

        Ok(symbols)
    }

    pub fn notify_file_changed(
        &mut self,
        file_uri: &str,
        code: &str,
        lang: &str,
    ) -> std::io::Result<()> {
        let lang_lower = lang.to_lowercase();
        let lsp_lang = match lang_lower.as_str() {
            "jsx" => "javascriptreact",
            "tsx" => "typescriptreact",
            "js" => "javascript",
            "ts" => "typescript",
            other => other,
        };
        if !self.opened_files.contains(file_uri) {
            let did_open = json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": file_uri,
                        "languageId": lsp_lang,
                        "version": 1,
                        "text": code
                    }
                }
            });
            self.send_notification(&did_open)?;
            self.opened_files.insert(file_uri.to_string());
            self.file_versions.insert(file_uri.to_string(), 1);
        } else {
            let version = self.file_versions.get(file_uri).unwrap_or(&1) + 1;
            let did_change = json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didChange",
                "params": {
                    "textDocument": {
                        "uri": file_uri,
                        "version": version
                    },
                    "contentChanges": [
                        {
                            "text": code
                        }
                    ]
                }
            });
            self.send_notification(&did_change)?;
            self.file_versions.insert(file_uri.to_string(), version);
        }
        Ok(())
    }

    pub fn notify_file_saved(&mut self, file_uri: &str) -> std::io::Result<()> {
        let did_save = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didSave",
            "params": {
                "textDocument": {
                    "uri": file_uri
                }
            }
        });
        self.send_notification(&did_save)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_diagnostics(&self, file_uri: &str) -> Vec<Diagnostic> {
        self.diagnostics
            .lock()
            .unwrap()
            .get(file_uri)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_all_diagnostics(&self, project_dir: &str) -> Vec<Diagnostic> {
        let diags_lock = self.diagnostics.lock().unwrap();
        let mut all = Vec::new();
        let project_dir_clean = project_dir.replace('\\', "/");
        let prefix = format!("file://{}/", project_dir_clean);
        let prefix_alt = format!("file://{}", project_dir_clean);
        for (uri, file_diags) in diags_lock.iter() {
            let uri_clean = uri.replace('\\', "/");
            let mut rel_path = if uri_clean.starts_with(&prefix) {
                uri_clean
                    .strip_prefix(&prefix)
                    .unwrap_or(&uri_clean)
                    .to_string()
            } else if uri_clean.starts_with(&prefix_alt) {
                uri_clean
                    .strip_prefix(&prefix_alt)
                    .unwrap_or(&uri_clean)
                    .to_string()
            } else {
                let p_with_file = "file://";
                if uri_clean.starts_with(p_with_file) {
                    uri_clean
                        .strip_prefix(p_with_file)
                        .unwrap_or(&uri_clean)
                        .to_string()
                } else {
                    uri_clean.clone()
                }
            };

            if rel_path.starts_with('/') {
                rel_path = rel_path.trim_start_matches('/').to_string();
            }

            for d in file_diags {
                let mut d_clone = d.clone();
                d_clone.file = Some(rel_path.clone());
                all.push(d_clone);
            }
        }
        all
    }

    pub fn get_diagnostics_version(&self, file_uri: &str) -> usize {
        self.diagnostics_version
            .lock()
            .unwrap()
            .get(file_uri)
            .cloned()
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct CompletionItem {
    pub label: String,
    pub insert_text: Option<String>,
    pub kind: Option<u32>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
}

impl Eq for CompletionItem {}

impl PartialOrd for CompletionItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.label.cmp(&other.label))
    }
}

impl Ord for CompletionItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.label.cmp(&other.label)
    }
}

pub fn fallback_completions(code: &str, prefix: &str) -> Vec<CompletionItem> {
    let mut words = std::collections::HashSet::new();
    // Match words that are at least 2 characters long
    let re = regex::Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]+\b").unwrap();
    for cap in re.captures_iter(code) {
        let w = cap[0].to_string();
        // If we have a prefix, only include words that start with it
        if prefix.is_empty() || (w.starts_with(prefix) && w != prefix) {
            words.insert(w);
        }
    }
    let mut res: Vec<CompletionItem> = words
        .into_iter()
        .map(|w| CompletionItem {
            label: w,
            insert_text: None,
            kind: Some(6), // Variable/Text kind
            detail: None,
            documentation: None,
        })
        .collect();
    res.sort();
    res
}

fn parse_location(val: &Value) -> Option<Location> {
    if let (Some(uri), Some(range_val)) = (
        val.get("uri")
            .and_then(|u| u.as_str())
            .or_else(|| val.get("targetUri").and_then(|u| u.as_str())),
        val.get("range").or_else(|| val.get("targetRange")),
    ) {
        if let Ok(range) = serde_json::from_value::<Range>(range_val.clone()) {
            return Some(Location {
                uri: uri.to_string(),
                range,
            });
        }
    }
    None
}

fn parse_symbol_item(
    val: &Value,
    list: &mut Vec<DocumentSymbolResponse>,
    container_name: Option<String>,
) {
    if let Some(name) = val["name"].as_str() {
        let kind = val["kind"].as_u64().unwrap_or(0) as u32;

        let (line, character) = if let Some(range_val) = val.get("range") {
            let start = &range_val["start"];
            (
                start["line"].as_u64().unwrap_or(0) as u32,
                start["character"].as_u64().unwrap_or(0) as u32,
            )
        } else if let Some(loc_val) = val.get("location") {
            let start = &loc_val["range"]["start"];
            (
                start["line"].as_u64().unwrap_or(0) as u32,
                start["character"].as_u64().unwrap_or(0) as u32,
            )
        } else {
            (0, 0)
        };

        let current_container = val["containerName"]
            .as_str()
            .map(|s| s.to_string())
            .or(container_name.clone());

        list.push(DocumentSymbolResponse {
            name: name.to_string(),
            kind,
            line,
            character,
            container_name: current_container.clone(),
        });

        if let Some(children) = val["children"].as_array() {
            let next_container = if current_container.is_some() {
                format!("{}.{}", current_container.as_ref().unwrap(), name)
            } else {
                name.to_string()
            };
            for child in children {
                parse_symbol_item(child, list, Some(next_container.clone()));
            }
        }
    }
}

pub fn fallback_symbols(code: &str, lang: &str) -> Vec<DocumentSymbolResponse> {
    let mut symbols = Vec::new();
    let lang_lower = lang.to_lowercase();

    for (i, line) in code.lines().enumerate() {
        let line_trimmed = line.trim();
        if line_trimmed.is_empty() {
            continue;
        }

        match lang_lower.as_str() {
            "rust" => {
                if line_trimmed.starts_with("fn ") || line_trimmed.contains(" fn ") {
                    if let Some(name) = extract_rust_name(line_trimmed, "fn") {
                        symbols.push(DocumentSymbolResponse {
                            name,
                            kind: 12, // Function
                            line: i as u32,
                            character: line.find("fn").unwrap_or(0) as u32,
                            container_name: None,
                        });
                    }
                } else if line_trimmed.starts_with("struct ") || line_trimmed.contains(" struct ") {
                    if let Some(name) = extract_rust_name(line_trimmed, "struct") {
                        symbols.push(DocumentSymbolResponse {
                            name,
                            kind: 23, // Struct
                            line: i as u32,
                            character: line.find("struct").unwrap_or(0) as u32,
                            container_name: None,
                        });
                    }
                } else if line_trimmed.starts_with("enum ") || line_trimmed.contains(" enum ") {
                    if let Some(name) = extract_rust_name(line_trimmed, "enum") {
                        symbols.push(DocumentSymbolResponse {
                            name,
                            kind: 10, // Enum
                            line: i as u32,
                            character: line.find("enum").unwrap_or(0) as u32,
                            container_name: None,
                        });
                    }
                } else if line_trimmed.starts_with("impl") {
                    let parts: Vec<&str> = line_trimmed.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let impl_name = parts[1..]
                            .join(" ")
                            .trim_end_matches('{')
                            .trim()
                            .to_string();
                        symbols.push(DocumentSymbolResponse {
                            name: format!("impl {}", impl_name),
                            kind: 5, // Class/Implementation
                            line: i as u32,
                            character: 0,
                            container_name: None,
                        });
                    }
                } else if line_trimmed.starts_with("trait ") || line_trimmed.contains(" trait ") {
                    if let Some(name) = extract_rust_name(line_trimmed, "trait") {
                        symbols.push(DocumentSymbolResponse {
                            name,
                            kind: 11, // Interface/Trait
                            line: i as u32,
                            character: line.find("trait").unwrap_or(0) as u32,
                            container_name: None,
                        });
                    }
                }
            }
            "python" => {
                if line_trimmed.starts_with("def ") {
                    if let Some(name) = line_trimmed.strip_prefix("def ") {
                        let name = name.split('(').next().unwrap_or(name).trim().to_string();
                        let kind = if line.starts_with(' ') || line.starts_with('\t') {
                            6 // Method
                        } else {
                            12 // Function
                        };
                        symbols.push(DocumentSymbolResponse {
                            name,
                            kind,
                            line: i as u32,
                            character: (line.len() - line_trimmed.len()) as u32,
                            container_name: None,
                        });
                    }
                } else if line_trimmed.starts_with("class ") {
                    if let Some(name) = line_trimmed.strip_prefix("class ") {
                        let name = name
                            .split('(')
                            .next()
                            .unwrap_or(name)
                            .split(':')
                            .next()
                            .unwrap_or(name)
                            .trim()
                            .to_string();
                        symbols.push(DocumentSymbolResponse {
                            name,
                            kind: 5, // Class
                            line: i as u32,
                            character: (line.len() - line_trimmed.len()) as u32,
                            container_name: None,
                        });
                    }
                }
            }
            "javascript" | "typescript" | "jsx" | "tsx" => {
                if line_trimmed.starts_with("function ") || line_trimmed.contains(" function ") {
                    if let Some(name) = extract_js_name(line_trimmed, "function") {
                        symbols.push(DocumentSymbolResponse {
                            name,
                            kind: 12, // Function
                            line: i as u32,
                            character: line.find("function").unwrap_or(0) as u32,
                            container_name: None,
                        });
                    }
                } else if line_trimmed.starts_with("class ") || line_trimmed.contains(" class ") {
                    if let Some(name) = extract_js_name(line_trimmed, "class") {
                        symbols.push(DocumentSymbolResponse {
                            name,
                            kind: 5, // Class
                            line: i as u32,
                            character: line.find("class").unwrap_or(0) as u32,
                            container_name: None,
                        });
                    }
                } else if line_trimmed.starts_with("const ")
                    || line_trimmed.starts_with("let ")
                    || line_trimmed.starts_with("var ")
                {
                    if line_trimmed.contains("=>") {
                        let parts: Vec<&str> = line_trimmed.split('=').collect();
                        if !parts.is_empty() {
                            let decl = parts[0].trim();
                            let name = decl.split_whitespace().last().unwrap_or("").trim();
                            if !name.is_empty() {
                                symbols.push(DocumentSymbolResponse {
                                    name: name.to_string(),
                                    kind: 12, // Function (Arrow Function)
                                    line: i as u32,
                                    character: (line.len() - line_trimmed.len()) as u32,
                                    container_name: None,
                                });
                            }
                        }
                    }
                }
            }
            "go" => {
                if line_trimmed.starts_with("func ") {
                    let rest = &line_trimmed[5..];
                    if rest.starts_with('(') {
                        if let Some(close_paren_idx) = rest.find(')') {
                            let method_part = &rest[close_paren_idx + 1..].trim();
                            let name = method_part
                                .split('(')
                                .next()
                                .unwrap_or(method_part)
                                .trim()
                                .to_string();
                            symbols.push(DocumentSymbolResponse {
                                name,
                                kind: 6, // Method
                                line: i as u32,
                                character: (line.len() - line_trimmed.len()) as u32,
                                container_name: None,
                            });
                        }
                    } else {
                        let name = rest.split('(').next().unwrap_or(rest).trim().to_string();
                        symbols.push(DocumentSymbolResponse {
                            name,
                            kind: 12, // Function
                            line: i as u32,
                            character: (line.len() - line_trimmed.len()) as u32,
                            container_name: None,
                        });
                    }
                } else if line_trimmed.starts_with("type ") {
                    let rest = &line_trimmed[5..];
                    let parts: Vec<&str> = rest.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let name = parts[0];
                        let type_kind = parts[1];
                        let kind = if type_kind == "interface" {
                            11 // Interface
                        } else {
                            23 // Struct
                        };
                        symbols.push(DocumentSymbolResponse {
                            name: name.to_string(),
                            kind,
                            line: i as u32,
                            character: (line.len() - line_trimmed.len()) as u32,
                            container_name: None,
                        });
                    }
                }
            }
            _ => {}
        }
    }
    symbols
}

fn extract_rust_name(line: &str, keyword: &str) -> Option<String> {
    if let Some(idx) = line.find(keyword) {
        let after = &line[idx + keyword.len()..].trim();
        let name = after
            .split(|c: char| c == '<' || c == '(' || c == '{' || c == ':' || c == ';')
            .next()?
            .trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

fn extract_js_name(line: &str, keyword: &str) -> Option<String> {
    if let Some(idx) = line.find(keyword) {
        let after = &line[idx + keyword.len()..].trim();
        let name = after
            .split(|c: char| c == '(' || c == '{' || c == '<' || c == ' ' || c == ';')
            .next()?
            .trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}
