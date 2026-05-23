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
pub struct Diagnostic {
    pub range: Range,
    pub severity: Option<u32>,
    pub code: Option<serde_json::Value>,
    pub source: Option<String>,
    pub message: String,
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
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

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
                println!(" [{} LSP stderr] {}", lang_name, line.trim());
                line.clear();
            }
        });

        let responses_clone = responses.clone();
        let diagnostics_clone = diagnostics.clone();
        let diagnostics_version_clone = diagnostics_version.clone();

        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut line = String::new();
                let mut content_length = 0;
                while let Ok(len) = reader.read_line(&mut line) {
                    if len == 0 {
                        return;
                    } // EOF
                    if line == "\r\n" {
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
                            if let Some(id) = val.get("id").and_then(|id| id.as_u64()) {
                                responses_clone.lock().unwrap().insert(id as usize, val);
                            } else if let Some(method) = val.get("method").and_then(|m| m.as_str()) {
                                if method == "textDocument/publishDiagnostics" {
                                    if let Some(params) = val.get("params") {
                                        if let (Some(uri), Some(diags)) = (params["uri"].as_str(), params["diagnostics"].as_array()) {
                                            if let Ok(parsed_diags) = serde_json::from_value::<Vec<Diagnostic>>(json!(diags)) {
                                                diagnostics_clone.lock().unwrap().insert(uri.to_string(), parsed_diags);
                                                let mut versions = diagnostics_version_clone.lock().unwrap();
                                                let entry = versions.entry(uri.to_string()).or_insert(0);
                                                *entry += 1;
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

        let init_req = json!({
            "jsonrpc": "2.0",
            "id": client.req_id,
            "method": "initialize",
            "params": {
                "processId": std::process::id(),
                "rootUri": root_uri,
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
                        "definition": { "dynamicRegistration": true }
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
        let msg = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        let mut stdin = self.stdin.lock().unwrap();
        stdin.write_all(msg.as_bytes())?;
        stdin.flush()?;
        Ok(())
    }

    fn send_request(&mut self, req: &Value) -> std::io::Result<()> {
        let body = req.to_string();
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

    pub fn notify_file_changed(
        &mut self,
        file_uri: &str,
        code: &str,
        lang: &str,
    ) -> std::io::Result<()> {
        if !self.opened_files.contains(file_uri) {
            let did_open = json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": file_uri,
                        "languageId": lang,
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

    pub fn get_diagnostics(&self, file_uri: &str) -> Vec<Diagnostic> {
        self.diagnostics
            .lock()
            .unwrap()
            .get(file_uri)
            .cloned()
            .unwrap_or_default()
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
