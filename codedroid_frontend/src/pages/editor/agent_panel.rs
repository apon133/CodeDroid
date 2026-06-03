use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use gloo_storage::Storage;
use crate::api;
use crate::components::icon::LucideIcon;
use crate::pages::editor::utils::file_icon;
use crate::models::Settings;
use crate::store;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: String, // "user", "assistant", "system"
    pub content: String,
}

#[derive(Clone, PartialEq, Debug)]
enum ToolCall {
    ReadFile { path: String },
    ProposeDiff { path: String, new_content: String },
    WriteFile { path: String, content: String },
    ScanProject,
    RunCommand { command: String },
    DeleteFile { path: String },
}

const SYSTEM_PROMPT: &str = r#"You are Antigravity, a highly capable autonomous AI coding agent with 20 years of experience.
You can read files, write files, delete files, scan the project directory, run terminal commands, and propose code changes.
Use your tools to fulfill the user's objective.

To interact with the project, you must use one of the following tools by outputting the specific XML-like tags. Do not explain the tool call. Call only one tool at a time, then stop and wait for the system to execute the tool and return the output.

Available Tools:

1. Read a File:
<read_file>relative/path/to/file</read_file>

2. Propose Code Change (Diff) for an existing file:
To make edits to existing files, you MUST propose changes by replacing specific blocks of code. Use one or more SEARCH/REPLACE blocks inside `<propose_diff>`. This is highly efficient and avoids outputting the entire file.
Format:
<propose_diff path="relative/path/to/file">
<<<<<<< SEARCH
exact lines of original code to be replaced
=======
new lines of code replacing them
>>>>>>> REPLACE
</propose_diff>

Guidelines for SEARCH/REPLACE blocks:
- The SEARCH block must match the existing file contents EXACTLY, line-for-line, including all spaces, tabs, comments, and empty lines.
- Keep the SEARCH block as short as possible to only target the lines you want to change, but include enough surrounding context lines (usually 1-3 lines before/after) to make it unique within the file.
- If you need to make multiple edits in the same file, you can stack multiple SEARCH/REPLACE blocks inside a single `<propose_diff>` tag.
- If you want to insert code without deleting anything, make the SEARCH block have the lines where you want to insert it, and include those same lines in the REPLACE block with the new code inserted between them.
- To overwrite/write the entire file, you can output the entire file content without the SEARCH/REPLACE markers, but SEARCH/REPLACE is highly preferred for modifications of existing files.

3. Write/Create a New File:
<write_file path="relative/path/to/file">
file content here
</write_file>

4. Scan Project (lists all file paths in the workspace):
<scan_project />

5. Run Terminal Command (in the project root):
<run_command>appropriate build or test command for the project</run_command>

6. Delete File:
<delete_file>relative/path/to/file</delete_file>

Guidelines:
- Always propose changes using `<propose_diff>` for existing files. This displays a red/green diff to the user, allowing them to Accept or Reject your changes.
- Ensure paths are relative to the project root.
- Run one tool call per turn. Once you call a tool, stop and wait. The output of the tool will be provided to you as a System message.
- Ensure your XML tags are correctly formed (e.g. `<read_file>path</read_file>`). Do not omit closing angle brackets or mix attribute styles.

CRITICAL RULES FOR AUTONOMY:
1. YOU ARE AN AUTONOMOUS AGENT, NOT A CONVERSATIONAL CHAT ASSISTANT.
2. NEVER ASK THE USER FOR CODE, FILES, CONTEXT, OR ERROR MESSAGES. If the user tells you to "solve error", "solve all errors", or similar, do NOT reply asking the user to provide details. You must find the errors yourself.
3. HOW TO FIND AND SOLVE ERRORS AUTONOMOUSLY:
   - First, run `<scan_project />` to list all files in the project.
   - Detect the project type (e.g., if you see `package.json` it is Svelte/Vite/Node, if `Cargo.toml` it is Rust, if `pubspec.yaml` it is Dart/Flutter, etc.).
   - Run the appropriate build, compiler, type-check, or test command for the project using `<run_command>` (e.g., `npm run build` or `npx tsc` for Svelte/Vite/TS/JS, `cargo check` for Rust, `dart analyze` for Flutter).
   - Read the output of the command to find exactly which file and line number have the error.
   - Use `<read_file>` to view only the relevant lines around the error in that file (do not read the entire file if it is very large).
   - Propose precise SEARCH/REPLACE blocks using `<propose_diff>` to fix the specific lines causing the error.
   - Run the build/check command again to verify that the error has been successfully resolved. Repeat until the build compiles with zero errors.
"#;

pub fn generate_line_diff(original: &str, new_text: &str) -> String {
    let orig_lines: Vec<&str> = original.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();
    let n = orig_lines.len();
    let m = new_lines.len();
    
    let mut dp = vec![vec![0; m + 1]; n + 1];
    for i in 1..=n {
        for j in 1..=m {
            if orig_lines[i-1] == new_lines[j-1] {
                dp[i][j] = dp[i-1][j-1] + 1;
            } else {
                dp[i][j] = dp[i-1][j].max(dp[i][j-1]);
            }
        }
    }
    
    let mut diff_lines = Vec::new();
    let mut i = n;
    let mut j = m;
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && orig_lines[i-1] == new_lines[j-1] {
            diff_lines.push(format!(" {}", orig_lines[i-1]));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j-1] >= dp[i-1][j]) {
            diff_lines.push(format!("+{}", new_lines[j-1]));
            j -= 1;
        } else if i > 0 && (j == 0 || dp[i-1][j] > dp[i][j-1]) {
            diff_lines.push(format!("-{}", orig_lines[i-1]));
            i -= 1;
        }
    }
    diff_lines.reverse();
    diff_lines.join("\n")
}

pub fn apply_search_replace(original: &str, edit_block: &str) -> Result<String, String> {
    let mut current_text = original.replace("\r\n", "\n");
    let normalized_edit = edit_block.replace("\r\n", "\n");

    let has_search_marker = normalized_edit.contains("<<<<<<< SEARCH") 
        || normalized_edit.contains("<search>") 
        || normalized_edit.contains("<SEARCH>");
        
    if !has_search_marker {
        // Fallback: it's the entire new content
        return Ok(edit_block.to_string());
    }

    let mut last_pos = 0;
    
    let find_first_of = |text: &str, patterns: &[&'static str], start: usize| -> Option<(usize, &'static str)> {
        let mut min_pos = None;
        let mut chosen_pattern = "";
        for &pattern in patterns {
            if let Some(pos) = text[start..].find(pattern) {
                let abs_pos = start + pos;
                if min_pos.is_none() || abs_pos < min_pos.unwrap() {
                    min_pos = Some(abs_pos);
                    chosen_pattern = pattern;
                }
            }
        }
        min_pos.map(move |pos| (pos, chosen_pattern))
    };

    let start_markers = &["<<<<<<< SEARCH", "<search>", "<SEARCH>"];
    let boundary_markers = &["=======", "<<<<<<< REPLACE", "<replace>", "<REPLACE>", "</search>", "</SEARCH>"];
    let end_markers = &[">>>>>>> REPLACE", "</replace>", "</REPLACE>"];

    while let Some((start_idx, start_marker)) = find_first_of(&normalized_edit, start_markers, last_pos) {
        let search_start_content = start_idx + start_marker.len();
        
        let Some((boundary_idx, boundary_marker)) = find_first_of(&normalized_edit, boundary_markers, search_start_content) else {
            return Err("Malformed edit block: could not find boundary (like ======= or <replace>) after search block start.".to_string());
        };
        
        let mut replace_start_content = boundary_idx + boundary_marker.len();
        
        // Skip optional intermediate markers
        let mut temp_pos = replace_start_content;
        while temp_pos < normalized_edit.len() {
            let next_char = normalized_edit[temp_pos..].chars().next().unwrap_or(' ');
            if next_char.is_whitespace() {
                temp_pos += next_char.len_utf8();
            } else if normalized_edit[temp_pos..].starts_with("<replace>") {
                temp_pos += "<replace>".len();
                replace_start_content = temp_pos;
            } else if normalized_edit[temp_pos..].starts_with("<REPLACE>") {
                temp_pos += "<REPLACE>".len();
                replace_start_content = temp_pos;
            } else if normalized_edit[temp_pos..].starts_with("<<<<<<< REPLACE") {
                temp_pos += "<<<<<<< REPLACE".len();
                replace_start_content = temp_pos;
            } else {
                break;
            }
        }

        let end_idx = match find_first_of(&normalized_edit, end_markers, replace_start_content) {
            Some((pos, _)) => pos,
            None => {
                match find_first_of(&normalized_edit, start_markers, replace_start_content) {
                    Some((pos, _)) => pos,
                    None => normalized_edit.len()
                }
            }
        };

        let mut search_content = &normalized_edit[search_start_content..boundary_idx];
        let mut replace_content = &normalized_edit[replace_start_content..end_idx];

        if search_content.starts_with('\n') {
            search_content = &search_content[1..];
        }
        if search_content.ends_with('\n') {
            search_content = &search_content[..search_content.len() - 1];
        }
        
        if replace_content.starts_with('\n') {
            replace_content = &replace_content[1..];
        }
        if replace_content.ends_with('\n') {
            replace_content = &replace_content[..replace_content.len() - 1];
        }

        if let Some(pos) = current_text.find(search_content) {
            current_text.replace_range(pos..pos + search_content.len(), replace_content);
        } else {
            let trimmed_search = search_content.trim();
            if !trimmed_search.is_empty() {
                if let Some(pos) = current_text.find(trimmed_search) {
                    current_text.replace_range(pos..pos + trimmed_search.len(), replace_content.trim());
                    last_pos = end_idx + 1;
                    continue;
                }
            }

            return Err(format!(
                "Could not find the SEARCH block in the file. Make sure the SEARCH block matches the file content EXACTLY (including whitespace and comments).\n\nSEARCH block:\n```\n{}\n```",
                search_content
            ));
        }

        last_pos = end_idx + 1;
    }

    Ok(current_text)
}

fn sanitize_path(path: &str) -> String {
    path.chars()
        .filter(|&c| c != '"' && c != '\'' && c != '>' && c != '<')
        .collect::<String>()
        .trim()
        .to_string()
}

fn extract_tag_value(content: &str, tag_name: &str) -> Option<String> {
    let open_tag = format!("<{}", tag_name);
    
    if let Some(start_idx) = content.find(&open_tag) {
        let sub = &content[start_idx..];
        
        // Case 1: Self-closing tag like <read_file path="src/main.rs" />
        if let Some(slash_idx) = sub.find("/>") {
            let tag_content = &sub[open_tag.len()..slash_idx];
            for attr in &["path=\"", "cmd=\"", "command=\""] {
                if let Some(attr_idx) = tag_content.find(attr) {
                    let after_attr = &tag_content[attr_idx + attr.len()..];
                    if let Some(quote_end) = after_attr.find('"') {
                        return Some(sanitize_path(&after_attr[..quote_end]));
                    }
                }
            }
        }
        
        // Case 2: Matching close tag or generic close tags
        let close_tag = format!("</{}>", tag_name);
        let mut end_idx_in_sub = None;
        for potential_end in &[&close_tag, "</arg_value>", "</tool_call>"] {
            if let Some(idx) = sub.find(potential_end) {
                if end_idx_in_sub.is_none() || idx < end_idx_in_sub.unwrap() {
                    end_idx_in_sub = Some(idx);
                }
            }
        }
        
        if let Some(end_idx) = end_idx_in_sub {
            let tag_and_body = &sub[..end_idx];
            if let Some(open_tag_end) = tag_and_body.find('>') {
                let tag_header = &tag_and_body[..open_tag_end];
                let body = &tag_and_body[open_tag_end + 1..];
                
                for attr_prefix in &["path=", "cmd=", "command="] {
                    if let Some(attr_idx) = tag_header.find(attr_prefix) {
                        let after_attr = &tag_header[attr_idx + attr_prefix.len()..];
                        let first_char = after_attr.chars().next();
                        let (start_offset, end_char) = match first_char {
                            Some('"') => (1, '"'),
                            Some('\'') => (1, '\''),
                            _ => (0, ' '),
                        };
                        let val_part = &after_attr[start_offset..];
                        let val_len = if end_char == ' ' {
                            val_part.find(|c| c == ' ' || c == '>').unwrap_or(val_part.len())
                        } else {
                            val_part.find(end_char).unwrap_or_else(|| {
                                val_part.find(|c| c == ' ' || c == '>').unwrap_or(val_part.len())
                            })
                        };
                        return Some(sanitize_path(&val_part[..val_len]));
                    }
                }
                
                let mut body_str = body;
                let trimmed = body_str.trim_start();
                if trimmed.starts_with("path=") || trimmed.starts_with("cmd=") || trimmed.starts_with("command=") {
                    if let Some(next_gt) = body_str.find('>') {
                        body_str = &body_str[next_gt + 1..];
                    }
                }
                
                return Some(body_str.trim().to_string());
            } else {
                // Malformed opening tag like <read_file src/main.rs
                let body = tag_and_body[open_tag.len()..].trim();
                return Some(body.trim().to_string());
            }
        }
    }
    None
}

fn strip_markdown_code_block(content: &str) -> String {
    let mut content = content.trim().to_string();

    // Strip leaked closing think tags at the end
    if content.ends_with("</think>") {
        content = content[..content.len() - "</think>".len()].trim().to_string();
    }

    // Strip leaked opening and closing think tags at the start/middle
    if content.starts_with("<think>") {
        if let Some(think_end) = content.find("</think>") {
            content = content[think_end + "</think>".len()..].trim().to_string();
        }
    }

    // Strip markdown code block wrappers
    if content.starts_with("```") {
        if let Some(first_newline) = content.find('\n') {
            let after_first_line = &content[first_newline + 1..];
            if after_first_line.ends_with("```") {
                content = after_first_line[..after_first_line.len() - 3].trim().to_string();
            } else {
                content = after_first_line.trim().to_string();
            }
        }
    } else if content.ends_with("```") {
        content = content[..content.len() - 3].trim().to_string();
    }

    // Check again just in case thinking tag was inside the markdown block
    if content.ends_with("</think>") {
        content = content[..content.len() - "</think>".len()].trim().to_string();
    }

    content
}

fn parse_tool_call(content: &str) -> Option<ToolCall> {
    // 1. Propose Diff (highest priority as it contains multiline file contents)
    if let Some(start_idx) = content.find("<propose_diff") {
        let sub = &content[start_idx..];
        let mut path = None;
        for quote_style in &["path=\"", "path='", "path="] {
            if let Some(p_idx) = sub.find(quote_style) {
                let after_path = &sub[p_idx + quote_style.len()..];
                let end_char = if quote_style.ends_with('"') {
                    '"'
                } else if quote_style.ends_with('\'') {
                    '\''
                } else {
                    ' '
                };
                
                let path_len = if end_char == ' ' {
                    after_path.find(|c| c == ' ' || c == '>').unwrap_or(after_path.len())
                } else {
                    after_path.find(end_char).unwrap_or_else(|| {
                        after_path.find(|c| c == ' ' || c == '>').unwrap_or(after_path.len())
                    })
                };
                let raw_path = after_path[..path_len].to_string();
                path = Some(sanitize_path(&raw_path));
                break;
            }
        }
        
        if let Some(path) = path {
            if let Some(open_tag_end) = sub.find('>') {
                let mut body = &sub[open_tag_end + 1..];
                let trimmed = body.trim_start();
                if trimmed.starts_with("path=") {
                    if let Some(next_gt) = body.find('>') {
                        body = &body[next_gt + 1..];
                    }
                }
                
                let mut end_idx = body.len();
                for potential_end in &["</propose_diff>", "</arg_value>", "</tool_call>"] {
                    if let Some(idx) = body.find(potential_end) {
                        end_idx = end_idx.min(idx);
                    }
                }
                let new_content = strip_markdown_code_block(&body[..end_idx]);
                return Some(ToolCall::ProposeDiff { path, new_content });
            }
        }
    }

    // 2. Write File
    if let Some(start_idx) = content.find("<write_file") {
        let sub = &content[start_idx..];
        let mut path = None;
        for quote_style in &["path=\"", "path='", "path="] {
            if let Some(p_idx) = sub.find(quote_style) {
                let after_path = &sub[p_idx + quote_style.len()..];
                let end_char = if quote_style.ends_with('"') {
                    '"'
                } else if quote_style.ends_with('\'') {
                    '\''
                } else {
                    ' '
                };
                
                let path_len = if end_char == ' ' {
                    after_path.find(|c| c == ' ' || c == '>').unwrap_or(after_path.len())
                } else {
                    after_path.find(end_char).unwrap_or_else(|| {
                        after_path.find(|c| c == ' ' || c == '>').unwrap_or(after_path.len())
                    })
                };
                let raw_path = after_path[..path_len].to_string();
                path = Some(sanitize_path(&raw_path));
                break;
            }
        }
        
        if let Some(path) = path {
            if let Some(open_tag_end) = sub.find('>') {
                let mut body = &sub[open_tag_end + 1..];
                let trimmed = body.trim_start();
                if trimmed.starts_with("path=") {
                    if let Some(next_gt) = body.find('>') {
                        body = &body[next_gt + 1..];
                    }
                }
                
                let mut end_idx = body.len();
                for potential_end in &["</write_file>", "</arg_value>", "</tool_call>"] {
                    if let Some(idx) = body.find(potential_end) {
                        end_idx = end_idx.min(idx);
                    }
                }
                let content = strip_markdown_code_block(&body[..end_idx]);
                return Some(ToolCall::WriteFile { path, content });
            }
        }
    }

    // 3. Read File
    if let Some(path) = extract_tag_value(content, "read_file") {
        return Some(ToolCall::ReadFile { path });
    }

    // 4. Scan Project
    if content.contains("scan_project") {
        return Some(ToolCall::ScanProject);
    }

    // 5. Run Command
    if let Some(command) = extract_tag_value(content, "run_command") {
        return Some(ToolCall::RunCommand { command });
    }

    // 6. Delete File
    if let Some(path) = extract_tag_value(content, "delete_file") {
        return Some(ToolCall::DeleteFile { path });
    }

    None
}

async fn call_llm(
    project_path: &str,
    settings: &Settings,
    messages: Vec<ChatMessage>
) -> Result<String, String> {
    let url = if settings.ai_endpoint.contains("/api/v1/chat") {
        settings.ai_endpoint.clone()
    } else if settings.ai_endpoint.ends_with("/chat/completions") {
        settings.ai_endpoint.clone()
    } else {
        format!("{}/chat/completions", settings.ai_endpoint.trim_end_matches('/'))
    };

    let is_native_lm = url.contains("/api/v1/chat") && !url.contains("openrouter.ai") && settings.ai_provider != "openrouter";
    let body = if is_native_lm {
        let system_prompt = messages.iter().find(|m| m.role == "system").map(|m| m.content.clone()).unwrap_or_default();
        
        let mut conversation_text = String::new();
        let mut first_system = true;
        for msg in &messages {
            if msg.role == "system" {
                if first_system {
                    first_system = false;
                    continue;
                }
                conversation_text.push_str(&format!("System (Tool Output):\n{}\n\n", msg.content));
            } else {
                let role_name = match msg.role.as_str() {
                    "user" => "User",
                    "assistant" => "Assistant",
                    _ => "User",
                };
                conversation_text.push_str(&format!("{}:\n{}\n\n", role_name, msg.content));
            }
        }

        let input = if conversation_text.is_empty() {
            messages.last().map(|m| m.content.clone()).unwrap_or_default()
        } else {
            conversation_text
        };

        serde_json::json!({
            "model": settings.ai_model,
            "system_prompt": system_prompt,
            "input": input
        })
    } else {
        serde_json::json!({
            "model": settings.ai_model,
            "messages": messages,
            "temperature": 0.2
        })
    };

    let json_str = serde_json::to_string(&body).map_err(|e| format!("Failed to serialize request body: {}", e))?;
    
    let temp_filename = ".codedroid_temp_llm.json";
    let temp_out_filename = ".codedroid_temp_llm_out.json";
    let temp_status_filename = ".codedroid_temp_llm_status.txt";
    let temp_err_filename = ".codedroid_temp_llm_err.txt";

    let auth_header = if !settings.ai_api_key.is_empty() {
        format!("-H \"Authorization: Bearer {}\" ", settings.ai_api_key)
    } else {
        "".to_string()
    };

    // Use heredoc to write the JSON content directly inside the shell,
    // avoiding the backend sync_file API which has directory creation quirks.
    let combined_cmd = format!(
        "cat << 'EOF' > {}\n{}\nEOF\ncurl -s -X POST {} -H \"Content-Type: application/json\" -d @{} \"{}\" > {} 2> {} && echo done > {}",
        temp_filename,
        json_str,
        auth_header,
        temp_filename,
        url,
        temp_out_filename,
        temp_err_filename,
        temp_status_filename
    );

    // Spawn the shell command
    let cmd_res = api::run_command_api(project_path, &combined_cmd).await;

    let spawn_success = match &cmd_res {
        Ok(resp) => resp.success,
        Err(_) => false,
    };

    let mut response_content = None;
    let mut poll_error = None;

    if spawn_success {
        let mut attempts = 0;
        let max_attempts = 100; // 100 * 500ms = 50 seconds max wait
        let mut completed = false;

        let check_cmd = format!("cat {} 2>/dev/null || type {} 2>nul", temp_status_filename, temp_status_filename);

        while attempts < max_attempts {
            gloo_timers::future::TimeoutFuture::new(500).await;
            attempts += 1;

            if let Ok(resp) = api::run_command_api(project_path, &check_cmd).await {
                if resp.success && resp.output.trim() == "done" {
                    completed = true;
                    break;
                }
            }
        }

        if completed {
            let read_out_cmd = format!("cat {} 2>/dev/null || type {} 2>nul", temp_out_filename, temp_out_filename);
            match api::run_command_api(project_path, &read_out_cmd).await {
                Ok(resp) => {
                    if resp.success {
                        response_content = Some(resp.output);
                    } else {
                        poll_error = Some(format!("Failed to read LLM response: {}", resp.error));
                    }
                }
                Err(e) => {
                    poll_error = Some(format!("Failed to read LLM response: {}", e));
                }
            }
        } else {
            let read_err_cmd = format!("cat {} 2>/dev/null || type {} 2>nul", temp_err_filename, temp_err_filename);
            let mut err_msg = "LLM request timed out after 50 seconds.".to_string();
            if let Ok(resp) = api::run_command_api(project_path, &read_err_cmd).await {
                if resp.success && !resp.output.trim().is_empty() {
                    err_msg = format!("LLM request failed: {}", resp.output.trim());
                }
            }
            poll_error = Some(err_msg);
        }
    } else {
        poll_error = Some(match cmd_res {
            Ok(resp) => format!("Failed to spawn curl: {}. Stderr: {}", resp.output, resp.error),
            Err(e) => format!("Failed to spawn curl: {}", e),
        });
    }

    // Clean up temporary files using shell commands
    let cleanup_cmd = format!(
        "rm -f {} {} {} {} 2>/dev/null || del /f /q {} {} {} {} 2>nul",
        temp_filename, temp_out_filename, temp_err_filename, temp_status_filename,
        temp_filename, temp_out_filename, temp_err_filename, temp_status_filename
    );
    let _ = api::run_command_api(project_path, &cleanup_cmd).await;

    if let Some(err) = poll_error {
        return Err(err);
    }

    let stdout_text = response_content.unwrap_or_default();
    let stdout_text = stdout_text.trim();
    if stdout_text.is_empty() {
        return Err("Local LLM server returned empty response".to_string());
    }

    let json_val: serde_json::Value = serde_json::from_str(stdout_text)
        .map_err(|e| format!("Failed to parse JSON response: {}. Raw response: {}", e, stdout_text))?;

    if let Some(err_val) = json_val.get("error") {
        let err_msg = err_val.get("message")
            .and_then(|m| m.as_str())
            .unwrap_or_else(|| err_val.as_str().unwrap_or("Unknown error"));
        return Err(format!("LLM API returned error: {}", err_msg));
    }

    let content = if is_native_lm {
        json_val["output"][0]["content"]
            .as_str()
            .ok_or_else(|| format!("No output content found in native response: {}", json_val))?
    } else {
        json_val["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| format!("No message content found in OpenAI response: {}", json_val))?
    };

    Ok(content.to_string())
}

#[component]
pub fn AgentPanel(
    project_path: String,
    project_id: String,
    open_file: Callback<String>,
    show_snack: Callback<String>,
    close_sidebar: Callback<()>,
    refresh_files: Callback<()>,
) -> impl IntoView {
    let chat_history = RwSignal::new(Vec::<ChatMessage>::new());
    let input_value = RwSignal::new(String::new());
    let agent_status = RwSignal::new(String::new());
    let proposed_changes = RwSignal::new(Vec::<(String, String)>::new());
    let abort_requested = RwSignal::new(false);

    let resolved_path = RwSignal::new(project_path.clone());
    spawn_local({
        let project_path = project_path.clone();
        async move {
            if let Ok(resp) = api::run_command_api(&project_path, "pwd 2>/dev/null || cd").await {
                if resp.success {
                    resolved_path.set(resp.output.trim().to_string());
                }
            }
        }
    });

    // Load initial proposed changes from storage index if any
    Effect::new({
        let project_id = project_id.clone();
        move || {
            let key = format!("agent-proposed-list:{}", project_id);
            if let Ok(list) = gloo_storage::LocalStorage::get::<Vec<(String, String)>>(&key) {
                proposed_changes.set(list);
            }
        }
    });

    // Save proposed changes list to storage whenever it changes
    Effect::new({
        let project_id = project_id.clone();
        move || {
            let key = format!("agent-proposed-list:{}", project_id);
            let val = proposed_changes.get();
            let _ = gloo_storage::LocalStorage::set(&key, &val);
        }
    });

    let accept_change = {
        let resolved_path = resolved_path.clone();
        let project_id = project_id.clone();
        let open_file = open_file.clone();
        let show_snack = show_snack.clone();
        let refresh_files = refresh_files.clone();
        move |path: String, content: String| {
            let full_path = format!("{}/{}", resolved_path.get_untracked(), path);
            let path_clone = path.clone();
            let content_clone = content.clone();
            let open_file = open_file.clone();
            let show_snack = show_snack.clone();
            let project_id = project_id.clone();
            let refresh_files = refresh_files.clone();
            spawn_local(async move {
                match api::save_file_api(&full_path, &content_clone).await {
                    Ok(_) => {
                        // Update local storage editor cache for relative and absolute path
                        let rel_key = store::file_key(&project_id, &path_clone);
                        store::save_file(&rel_key, &content_clone);

                        let abs_key = store::file_key(&project_id, &full_path);
                        store::save_file(&abs_key, &content_clone);

                        // Clean up LocalStorage diff
                        let diff_key = format!("agent-diff:{}:{}", project_id, path_clone);
                        let _ = gloo_storage::LocalStorage::delete(&diff_key);

                        // Remove from proposed changes
                        proposed_changes.update(|list| {
                            list.retain(|(p, _)| p != &path_clone);
                        });

                        // Trigger file list refresh
                        refresh_files.run(());

                        show_snack.run(format!("Saved and applied changes to {}", path_clone));
                        
                        // Open the relative path in the editor
                        open_file.run(path_clone);
                    }
                    Err(e) => {
                        show_snack.run(format!("Failed to apply changes: {}", e));
                    }
                }
            });
        }
    };

    let reject_change = {
        let project_id = project_id.clone();
        let show_snack = show_snack.clone();
        move |path: String| {
            let diff_key = format!("agent-diff:{}:{}", project_id, path);
            let _ = gloo_storage::LocalStorage::delete(&diff_key);

            proposed_changes.update(|list| {
                list.retain(|(p, _)| p != &path);
            });

            show_snack.run(format!("Rejected changes for {}", path));
        }
    };

    // Inline action handlers are used below to avoid FnOnce constraint issues

    let run_agent_loop = {
        let resolved_path = resolved_path.clone();
        let project_id = project_id.clone();
        let open_file = open_file.clone();
        let show_snack = show_snack.clone();
        let refresh_files = refresh_files.clone();
        
        move || {
            let settings = store::load_settings();
            let mut history = chat_history.get_untracked();
            
            if history.is_empty() {
                return;
            }

            let resolved_path = resolved_path.clone();
            let project_id = project_id.clone();
            let open_file = open_file.clone();
            let show_snack = show_snack.clone();
            let refresh_files = refresh_files.clone();

            spawn_local(async move {
                abort_requested.set(false);
                agent_status.set("Thinking...".to_string());
                
                let resolved_path_str = resolved_path.get_untracked();
                let mut loop_count = 0;
                const MAX_LOOPS: usize = 10;

                while loop_count < MAX_LOOPS {
                    loop_count += 1;
                    
                    if abort_requested.get() {
                        history.push(ChatMessage {
                            role: "system".to_string(),
                            content: "Agent execution stopped by user.".to_string(),
                        });
                        chat_history.set(history.clone());
                        break;
                    }
                    
                    // Build the API message stack with workspace context
                    let mut system_prompt = SYSTEM_PROMPT.to_string();
                    system_prompt.push_str("\n\n=== WORKSPACE CONTEXT ===");
                    system_prompt.push_str(&format!("\nProject Root: {}", resolved_path_str));
                    system_prompt.push_str("\n=========================");

                    let mut api_messages = vec![
                        ChatMessage {
                            role: "system".to_string(),
                            content: system_prompt,
                        }
                    ];
                    api_messages.extend(history.clone());

                    let llm_res = call_llm(&resolved_path_str, &settings, api_messages).await;
                    if abort_requested.get() {
                        history.push(ChatMessage {
                            role: "system".to_string(),
                            content: "Agent execution stopped by user.".to_string(),
                        });
                        chat_history.set(history.clone());
                        break;
                    }

                    match llm_res {
                        Ok(ai_response) => {
                            // Append assistant's text
                            history.push(ChatMessage {
                                role: "assistant".to_string(),
                                content: ai_response.clone(),
                            });
                            chat_history.set(history.clone());

                            // Parse for tool calls
                            if let Some(tool) = parse_tool_call(&ai_response) {
                                match tool {
                                    ToolCall::ReadFile { path } => {
                                        agent_status.set(format!("Reading file: {}...", path));
                                        let full_path = format!("{}/{}", resolved_path_str, path);
                                        let result_msg = match api::read_file_api(&full_path).await {
                                            Ok(resp) => {
                                                if resp.error.is_empty() {
                                                    format!("File content for {}:\n```\n{}\n```", path, resp.content)
                                                } else {
                                                    format!("Error reading {}: {}", path, resp.error)
                                                }
                                            }
                                            Err(e) => format!("Failed to read file {}: {}", path, e),
                                        };
                                        history.push(ChatMessage {
                                            role: "system".to_string(),
                                            content: result_msg,
                                        });
                                        chat_history.set(history.clone());
                                    }
                                    ToolCall::ProposeDiff { path, new_content: edit_block } => {
                                        agent_status.set(format!("Proposing diff: {}...", path));
                                        let full_path = format!("{}/{}", resolved_path_str, path);
                                        
                                        // Read existing content to build line diff
                                        let current_content = match api::read_file_api(&full_path).await {
                                            Ok(resp) if resp.error.is_empty() => resp.content,
                                            _ => String::new(),
                                        };

                                        match apply_search_replace(&current_content, &edit_block) {
                                            Ok(new_content) => {
                                                let diff = generate_line_diff(&current_content, &new_content);
                                                
                                                // Store diff in LocalStorage
                                                let diff_key = format!("agent-diff:{}:{}", project_id, path);
                                                let _ = gloo_storage::LocalStorage::set(&diff_key, &diff);

                                                // Update proposed changes list
                                                proposed_changes.update(|list| {
                                                    list.retain(|(p, _)| p != &path);
                                                    list.push((path.clone(), new_content.clone()));
                                                });

                                                show_snack.run(format!("Diff proposed for {}!", path));
                                                
                                                // Open virtual diff view
                                                open_file.run(format!("agent-diff://{}", path));

                                                history.push(ChatMessage {
                                                    role: "system".to_string(),
                                                    content: format!("Successfully proposed diff for {}. Waiting for user's approval (Accept/Reject).", path),
                                                });
                                                chat_history.set(history.clone());
                                                break; // Pause loop to wait for user accept/reject
                                            }
                                            Err(err_msg) => {
                                                history.push(ChatMessage {
                                                    role: "system".to_string(),
                                                    content: format!("Error applying search/replace block: {}", err_msg),
                                                });
                                                chat_history.set(history.clone());
                                            }
                                        }
                                    }
                                    ToolCall::WriteFile { path, content } => {
                                        agent_status.set(format!("Writing file: {}...", path));
                                        let full_path = format!("{}/{}", resolved_path_str, path);
                                        let path_clone = path.clone();
                                        let content_clone = content.clone();
                                        let result_msg = match api::save_file_api(&full_path, &content_clone).await {
                                            Ok(_) => {
                                                // Sync editor cache
                                                let rel_key = store::file_key(&project_id, &path_clone);
                                                store::save_file(&rel_key, &content_clone);

                                                let abs_key = store::file_key(&project_id, &full_path);
                                                store::save_file(&abs_key, &content_clone);

                                                // Trigger file list refresh
                                                refresh_files.run(());

                                                // Open the newly written file
                                                open_file.run(path_clone);

                                                format!("Successfully created and saved file: {}", path)
                                            }
                                            Err(e) => format!("Failed to write file {}: {}", path, e),
                                        };
                                        history.push(ChatMessage {
                                            role: "system".to_string(),
                                            content: result_msg,
                                        });
                                        chat_history.set(history.clone());
                                    }
                                    ToolCall::ScanProject => {
                                        agent_status.set("Scanning directory...".to_string());
                                        let scan_res = api::scan_project_api(&resolved_path_str).await;
                                        if abort_requested.get() {
                                            history.push(ChatMessage {
                                                role: "system".to_string(),
                                                content: "Agent execution stopped by user.".to_string(),
                                            });
                                            chat_history.set(history.clone());
                                            break;
                                        }
                                        let result_msg = match scan_res {
                                            Ok(resp) if resp.error.is_empty() => {
                                                let paths: Vec<String> = resp.files.into_iter().map(|f| f.rel_path).collect();
                                                format!("Workspace scan result. Found {} files:\n{}", paths.len(), paths.join("\n"))
                                            }
                                            Ok(resp) => format!("Scan error: {}", resp.error),
                                            Err(e) => format!("Scan failed: {}", e),
                                        };
                                        history.push(ChatMessage {
                                            role: "system".to_string(),
                                            content: result_msg,
                                        });
                                        chat_history.set(history.clone());
                                    }
                                    ToolCall::RunCommand { command } => {
                                        agent_status.set(format!("Running command: {}...", command));
                                        let cmd_res = api::run_command_api(&resolved_path_str, &command).await;
                                        if abort_requested.get() {
                                            history.push(ChatMessage {
                                                role: "system".to_string(),
                                                content: "Agent execution stopped by user.".to_string(),
                                            });
                                            chat_history.set(history.clone());
                                            break;
                                        }
                                        let result_msg = match cmd_res {
                                            Ok(resp) => {
                                                format!("Command Output:\n```\n{}\n```\nError output:\n```\n{}\n```\nSuccess: {}", resp.output, resp.error, resp.success)
                                            }
                                            Err(e) => format!("Command failed to run: {}", e),
                                        };
                                        history.push(ChatMessage {
                                            role: "system".to_string(),
                                            content: result_msg,
                                        });
                                        chat_history.set(history.clone());
                                    }
                                    ToolCall::DeleteFile { path } => {
                                        agent_status.set(format!("Deleting file: {}...", path));
                                        let full_path = format!("{}/{}", resolved_path_str, path);
                                        let path_clone = path.clone();
                                        let result_msg = match api::delete_file_api(&full_path, false).await {
                                            Ok(_) => {
                                                // Clean up editor cache
                                                let rel_key = store::file_key(&project_id, &path_clone);
                                                let _ = gloo_storage::LocalStorage::delete(&rel_key);

                                                let abs_key = store::file_key(&project_id, &full_path);
                                                let _ = gloo_storage::LocalStorage::delete(&abs_key);

                                                // Trigger file list refresh
                                                refresh_files.run(());

                                                format!("Successfully deleted file: {}", path)
                                            }
                                            Err(e) => format!("Failed to delete file {}: {}", path, e),
                                        };
                                        history.push(ChatMessage {
                                            role: "system".to_string(),
                                            content: result_msg,
                                        });
                                        chat_history.set(history.clone());
                                    }
                                }
                            } else {
                                // No tool calls found - execution loop finished
                                break;
                            }
                        }
                        Err(e) => {
                            history.push(ChatMessage {
                                role: "system".to_string(),
                                content: format!("Error invoking AI: {}", e),
                            });
                            chat_history.set(history);
                            break;
                        }
                    }
                }
                
                agent_status.set(String::new());
            });
        }
    };

    let on_send = Callback::new({
        let run_agent_loop = run_agent_loop.clone();
        move |_: ()| {
            let val = input_value.get_untracked();
            if val.trim().is_empty() {
                return;
            }
            if !agent_status.get_untracked().is_empty() {
                return;
            }
            
            chat_history.update(|history| {
                history.push(ChatMessage {
                    role: "user".to_string(),
                    content: val,
                });
            });
            input_value.set(String::new());
            
            run_agent_loop();
        }
    });

    let on_clear = move || {
        chat_history.set(Vec::new());
        agent_status.set(String::new());
    };

    view! {
        <div class="sidebar-panel" style="display:flex; flex-direction:column; height:100%; background:var(--bg-sidebar); border-left:1px solid var(--border); width:var(--agent-width, 360px); flex-shrink:0">
            <style>
                "@keyframes agent-spin {
                    to { transform: rotate(360deg); }
                }"
            </style>
            // Panel Header
            <div class="sidebar-header" style="display:flex; align-items:center; justify-content:space-between; padding:10px 14px; border-bottom:1px solid var(--border)">
                <div style="display:flex; align-items:center; gap:8px; font-weight:600; color:var(--text)">
                    <LucideIcon name="sparkles" size="18" class="text-accent" />
                    <span>"Antigravity AI"</span>
                </div>
                <div style="display:flex; gap:6px">
                    <button class="btn btn-icon" title="Clear History" on:click=move |_| on_clear()>
                        <LucideIcon name="trash" size="16" />
                    </button>
                    <button class="btn btn-icon" title="Close Panel" on:click=move |_| close_sidebar.run(())>
                        <LucideIcon name="x" size="16" />
                    </button>
                </div>
            </div>

            // Main scrollable content (Chat Log + Proposed Diffs)
            <div class="sidebar-content" style="flex:1; overflow-y:auto; padding:12px; display:flex; flex-direction:column; gap:12px">
                
                // Chat History
                <div style="display:flex; flex-direction:column; gap:10px">
                    {move || {
                        let history = chat_history.get();
                        if history.is_empty() {
                            view! {
                                <div style="text-align:center; color:var(--text2); padding:24px 10px; font-size:13px">
                                    <span style="color:var(--accent); margin-bottom:8px; opacity:0.6; display:inline-block"><LucideIcon name="sparkles" size="32" /></span>
                                    <p style="font-weight:500; margin-bottom:4px">"Ask Antigravity anything!"</p>
                                    <span style="font-size:11px">"e.g., 'Find all tests in the project', 'Implement a login form', or 'Add a delete button to explorer'."</span>
                                </div>
                            }.into_any()
                        } else {
                            history.into_iter().map(|msg| {
                                let (bubble_class, bubble_style, label) = match msg.role.as_str() {
                                    "user" => (
                                        "chat-bubble user",
                                        "align-self: flex-end; background: linear-gradient(135deg, var(--accent) 0%, #a855f7 100%); color:#fff; border-radius: 12px 12px 2px 12px; max-width:85%; padding:8px 12px; font-size:13px; word-break:break-word; margin-left:auto;",
                                        "You"
                                    ),
                                    "assistant" => (
                                        "chat-bubble assistant",
                                        "align-self: flex-start; background: var(--bg-active); border: 1px solid var(--border); color:var(--text); border-radius: 12px 12px 12px 2px; max-width:85%; padding:8px 12px; font-size:13px; word-break:break-word;",
                                        "Antigravity"
                                    ),
                                    _ => (
                                        "chat-bubble system",
                                        "align-self: center; background: rgba(255,255,255,0.03); border: 1px dashed var(--border); color:var(--text2); border-radius: 6px; width:100%; padding:6px 10px; font-size:11px; font-family: monospace; white-space: pre-wrap; word-break:break-all;",
                                        "System"
                                    )
                                };
                                view! {
                                    <div style="display:flex; flex-direction:column; gap:4px">
                                        <span style=move || if msg.role == "user" { "font-size:10px; color:var(--text2); font-weight:600; text-align:right" } else { "font-size:10px; color:var(--text2); font-weight:600; text-align:left" }>{label}</span>
                                        <div class=bubble_class style=bubble_style>
                                            {msg.content}
                                        </div>
                                    </div>
                                }
                            }).collect_view().into_any()
                        }
                    }}
                </div>

                // Proposed Changes Section
                {move || {
                    let list = proposed_changes.get();
                    if list.is_empty() {
                        view! {}.into_any()
                    } else {
                        view! {
                            <div style="margin-top:16px; border-top:1px solid var(--border); padding-top:16px">
                                <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:8px">
                                    <span style="font-size:12px; font-weight:700; color:var(--text); display:flex; align-items:center; gap:6px">
                                        <LucideIcon name="edit" size="14" class="text-accent" />
                                        "Proposed Diffs"
                                        <span style="background:var(--accent); color:#000; border-radius:10px; padding:1px 6px; font-size:10px; font-weight:800">{list.len()}</span>
                                    </span>
                                    <div style="display:flex; gap:6px">
                                        <button class="btn btn-secondary" style="font-size:10px; padding:3px 8px; border-radius:4px" 
                                            on:click={
                                                let accept_change = accept_change.clone();
                                                move |_| {
                                                    let current = proposed_changes.get_untracked();
                                                    for (path, content) in current {
                                                        accept_change(path.clone(), content);
                                                    }
                                                }
                                            }>
                                            "Accept All"
                                        </button>
                                        <button class="btn" style="font-size:10px; padding:3px 8px; border-radius:4px; border:1px solid var(--border)" 
                                            on:click={
                                                let reject_change = reject_change.clone();
                                                move |_| {
                                                    let current = proposed_changes.get_untracked();
                                                    for (path, _) in current {
                                                        reject_change(path.clone());
                                                    }
                                                }
                                            }>
                                            "Reject All"
                                        </button>
                                    </div>
                                </div>

                                <div style="display:flex; flex-direction:column; gap:6px">
                                    {list.into_iter().map(|(path, content)| {
                                        let path_label = path.clone();
                                        let path_icon = path.clone();
                                        let path_accept = path.clone();
                                        let path_reject = path.clone();
                                        let content_clone = content.clone();
                                        let accept_cb = accept_change.clone();
                                        let reject_cb = reject_change.clone();
                                        let open_diff = open_file.clone();
                                        
                                        view! {
                                            <div style="display:flex; align-items:center; justify-content:space-between; background:var(--bg-active); border:1px solid var(--border); border-radius:6px; padding:6px 10px; gap:8px">
                                                <div style="display:flex; align-items:center; gap:6px; cursor:pointer; overflow:hidden; flex:1"
                                                    on:click=move |_| open_diff.run(format!("agent-diff://{}", path_label))>
                                                    <span style="opacity:0.8">{file_icon(&path_icon)}</span>
                                                    <span style="font-size:12px; color:var(--text); text-overflow:ellipsis; overflow:hidden; white-space:nowrap; font-weight:500">{path.clone()}</span>
                                                </div>
                                                <div style="display:flex; gap:4px">
                                                    <button class="btn btn-icon" style="color:#10b981" title="Accept"
                                                        on:click=move |_| accept_cb(path_accept.clone(), content_clone.clone())>
                                                        <LucideIcon name="check" size="14" />
                                                    </button>
                                                    <button class="btn btn-icon" style="color:#ef4444" title="Reject"
                                                        on:click=move |_| reject_cb(path_reject.clone())>
                                                        <LucideIcon name="x" size="14" />
                                                    </button>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            // Bottom Input Section
            <div style="padding:10px; border-top:1px solid var(--border); background:var(--bg-sidebar); display:flex; flex-direction:column; gap:8px">
                
                // Agent State Indicator
                {move || {
                    let status = agent_status.get();
                    if status.is_empty() {
                        view! {}.into_any()
                    } else {
                        view! {
                            <div style="display:flex; align-items:center; justify-content:space-between; width:100%; font-size:11px; color:var(--accent); font-weight:600">
                                <div style="display:flex; align-items:center; gap:6px">
                                    <span class="loader-spinner" style="width:12px; height:12px; border:2px solid var(--accent); border-top-color:transparent; border-radius:50%; display:inline-block; animation:agent-spin 1s linear infinite"></span>
                                    <span>{status}</span>
                                </div>
                                <button
                                    style="background:none; border:none; color:var(--error, #ef4444); cursor:pointer; font-size:10px; font-weight:700; display:flex; align-items:center; gap:4px; padding:2px 6px; border-radius:4px"
                                    on:click=move |_| abort_requested.set(true)
                                >
                                    <LucideIcon name="square" size="10" />
                                    <span>"Stop"</span>
                                </button>
                            </div>
                        }.into_any()
                    }
                }}

                <div style="display:flex; gap:6px; align-items:flex-end">
                    <textarea
                        class="input"
                        style="flex:1; min-height:40px; max-height:120px; font-size:13px; padding:8px; resize:none; border-radius:6px; background:var(--bg); color:var(--text); border:1px solid var(--border)"
                        placeholder="Ask Antigravity to edit code..."
                        prop:value=move || input_value.get()
                        on:input=move |e| input_value.set(event_target_value(&e))
                        on:keydown=move |e| {
                            let key = e.key();
                            if key == "Enter" && !e.shift_key() {
                                e.prevent_default();
                                if agent_status.get_untracked().is_empty() {
                                    on_send.run(());
                                }
                            }
                        }
                    />
                    {move || {
                        let status = agent_status.get();
                        if status.is_empty() {
                            view! {
                                <button
                                    class="btn"
                                    style="padding:10px 14px; background: linear-gradient(135deg, var(--accent) 0%, #a855f7 100%); color:#fff; font-weight:600; border-radius:6px; border:none; cursor:pointer; height:40px; display:flex; align-items:center; justify-content:center"
                                    on:click=move |_| on_send.run(())
                                >
                                    <LucideIcon name="arrow-up" size="18" />
                                </button>
                            }.into_any()
                        } else {
                            view! {
                                <button
                                    class="btn"
                                    style="padding:10px 14px; background: var(--error, #ef4444); color:#fff; font-weight:600; border-radius:6px; border:none; cursor:pointer; height:40px; display:flex; align-items:center; justify-content:center"
                                    on:click=move |_| abort_requested.set(true)
                                    title="Stop Execution"
                                >
                                    <LucideIcon name="square" size="18" />
                                </button>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
