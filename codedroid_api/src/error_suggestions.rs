use serde::{Deserialize, Serialize};
use axum::{Json, http::StatusCode};
use crate::lsp::{Diagnostic, Range, Position};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SuggestionRequest {
    pub code: String,
    pub language: String,
    pub diagnostic: Diagnostic,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeSuggestion {
    pub title: String,
    pub explanation: String,
    pub replacement: Option<String>,
    pub range: Option<Range>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SuggestionResponse {
    pub suggestions: Vec<CodeSuggestion>,
}

pub async fn get_error_suggestions_handler(
    Json(payload): Json<SuggestionRequest>,
) -> Result<Json<SuggestionResponse>, (StatusCode, String)> {
    let mut suggestions = Vec::new();
    let code = &payload.code;
    let diag = &payload.diagnostic;
    let msg = diag.message.to_lowercase();

    // ── RUST RULES ──
    if payload.language.to_lowercase() == "rust" {
        // Rule 1: Immutable mutation (E0384) or cannot borrow as mutable
        if msg.contains("cannot mutate immutable variable") 
            || msg.contains("cannot borrow") && msg.contains("as mutable") 
            || msg.contains("cannot assign to immutable") 
        {
            // Try to extract variable name from message: e.g. `cannot mutate immutable variable `x``
            let var_name = extract_variable_name(&diag.message);
            if let Some(ref name) = var_name {
                let explanation = format!(
                    "In Rust, variables are immutable by default. To mutate `{name}`, you must declare it with `let mut {name}` instead of `let {name}`."
                );
                
                // Let's try to locate the definition of this variable in previous lines to provide a replacement range
                if let Some(def_range) = find_let_binding_range(code, name, diag.range.start.line) {
                    suggestions.push(CodeSuggestion {
                        title: format!("Make `{name}` mutable"),
                        explanation,
                        replacement: Some(format!("let mut {name}")),
                        range: Some(def_range),
                    });
                } else {
                    suggestions.push(CodeSuggestion {
                        title: format!("Make `{name}` mutable"),
                        explanation,
                        replacement: None,
                        range: None,
                    });
                }
            } else {
                suggestions.push(CodeSuggestion {
                    title: "Make variable mutable".to_string(),
                    explanation: "Declare the variable with `let mut` to make it mutable.".to_string(),
                    replacement: None,
                    range: None,
                });
            }
        }
        
        // Rule 2: Unused variables
        if msg.contains("unused variable") {
            let var_name = extract_variable_name(&diag.message);
            if let Some(ref name) = var_name {
                suggestions.push(CodeSuggestion {
                    title: format!("Prefix unused variable `{name}` with underscore"),
                    explanation: "Prefixing variable name with `_` silences compiler warnings for unused variables in Rust.".to_string(),
                    replacement: Some(format!("_{name}")),
                    range: Some(diag.range.clone()),
                });
            }
        }

        // Rule 3: Mismatched types (expected XYZ, found ZYX)
        if msg.contains("mismatched types") || (msg.contains("expected") && msg.contains("found")) {
            // expected `String`, found `&str`
            if msg.contains("expected") && msg.contains("string") && msg.contains("found") && msg.contains("&str") {
                suggestions.push(CodeSuggestion {
                    title: "Convert `&str` to `String` using `.to_string()`".to_string(),
                    explanation: "The function expects an owned `String` but found a borrowed `&str`. You can convert it using `.to_string()` or `.into()`.".to_string(),
                    replacement: Some(format!("{}.to_string()", get_code_at_range(code, &diag.range))),
                    range: Some(diag.range.clone()),
                });
            }
            // expected `&str`, found `String`
            else if msg.contains("expected") && msg.contains("&str") && msg.contains("found") && msg.contains("string") {
                suggestions.push(CodeSuggestion {
                    title: "Borrow String as `&str` using `&`".to_string(),
                    explanation: "The function expects a borrowed string slice `&str` but found an owned `String`. You can borrow it using `&` or `.as_str()`.".to_string(),
                    replacement: Some(format!("&{}", get_code_at_range(code, &diag.range))),
                    range: Some(diag.range.clone()),
                });
            }
            // expected integer type mismatch: e.g. u32 and usize
            else if msg.contains("usize") && msg.contains("u32") {
                let code_snippet = get_code_at_range(code, &diag.range);
                suggestions.push(CodeSuggestion {
                    title: "Cast type with `as`".to_string(),
                    explanation: "Mismatched integer types. Cast the expression using `as u32` or `as usize`.".to_string(),
                    replacement: Some(format!("{} as _", code_snippet)),
                    range: Some(diag.range.clone()),
                });
            }
        }

        // Rule 4: Unresolved import / cannot find struct/trait/type/value in scope
        if msg.contains("cannot find") && (msg.contains("struct") || msg.contains("trait") || msg.contains("type") || msg.contains("value") || msg.contains("macro")) {
            let name = extract_unresolved_name(&diag.message);
            if let Some(ref n) = name {
                // Check if it's standard collections or Leptos traits/signals
                let suggestion_import = match n.as_str() {
                    "HashMap" => Some("use std::collections::HashMap;"),
                    "HashSet" => Some("use std::collections::HashSet;"),
                    "RwSignal" | "Signal" | "create_signal" | "component" | "IntoView" => Some("use leptos::prelude::*;"),
                    "Arc" => Some("use std::sync::Arc;"),
                    "Mutex" => Some("use std::sync::Mutex;"),
                    "Instant" => Some("use std::time::Instant;"),
                    "Duration" => Some("use std::time::Duration;"),
                    _ => None,
                };

                if let Some(import_stmt) = suggestion_import {
                    suggestions.push(CodeSuggestion {
                        title: format!("Import `{n}`"),
                        explanation: format!("Insert the import statement `{import_stmt}` at the top of the file to bring `{n}` into scope."),
                        replacement: Some(format!("{}\n", import_stmt)),
                        range: Some(Range {
                            start: Position { line: 0, character: 0 },
                            end: Position { line: 0, character: 0 },
                        }),
                    });
                }
            }
        }

        // Rule 5: Cannot move out of shared reference / value moved here
        if msg.contains("cannot move out of") && msg.contains("reference") {
            let code_snippet = get_code_at_range(code, &diag.range);
            suggestions.push(CodeSuggestion {
                title: "Clone the value to prevent move".to_string(),
                explanation: "The value cannot be moved out of a reference. Cloning creates an owned copy of the data.".to_string(),
                replacement: Some(format!("{}.clone()", code_snippet)),
                range: Some(diag.range.clone()),
            });
        }
    }

    // ── PYTHON RULES ──
    if payload.language.to_lowercase() == "python" {
        if msg.contains("indentationerror") || msg.contains("unexpected indent") {
            suggestions.push(CodeSuggestion {
                title: "Fix Indentation".to_string(),
                explanation: "Python is indentation-sensitive. Ensure that you use consistent spacing (4 spaces per indent level) and avoid mixing tabs and spaces.".to_string(),
                replacement: None,
                range: Some(diag.range.clone()),
            });
        }
        if msg.contains("nameerror") {
            let var_name = extract_variable_name(&diag.message);
            if let Some(ref name) = var_name {
                suggestions.push(CodeSuggestion {
                    title: format!("Check spelling or define `{name}`"),
                    explanation: format!("The name `{name}` is not defined in the scope. Check if you have defined it or spelled it correctly."),
                    replacement: None,
                    range: Some(diag.range.clone()),
                });
            }
        }
    }

    // ── JAVASCRIPT / TYPESCRIPT RULES ──
    if ["javascript", "typescript", "typescriptreact", "javascriptreact"].contains(&payload.language.to_lowercase().as_str()) {
        if msg.contains("cannot find name") {
            let name = extract_variable_name(&diag.message);
            if let Some(ref n) = name {
                suggestions.push(CodeSuggestion {
                    title: format!("Check spelling or import `{n}`"),
                    explanation: format!("Variable or symbol `{n}` cannot be found. Verify spelling, declaration, or import state."),
                    replacement: None,
                    range: Some(diag.range.clone()),
                });
            }
        }
    }

    // Default Fallback
    if suggestions.is_empty() {
        suggestions.push(CodeSuggestion {
            title: "Check code details and syntax".to_string(),
            explanation: format!("Review the diagnostic message: \"{}\". Ensure all imports, types, syntax tokens, and scope declarations are correct.", diag.message),
            replacement: None,
            range: None,
        });
    }

    Ok(Json(SuggestionResponse { suggestions }))
}

// Helper: Extract variable name in backticks: e.g. `x` or `my_var`
fn extract_variable_name(message: &str) -> Option<String> {
    let mut parts = message.split('`');
    // Skip the first part, take the second
    parts.next();
    parts.next().map(|s| s.to_string())
}

// Helper: Extract name from messages like: cannot find value `foo`
fn extract_unresolved_name(message: &str) -> Option<String> {
    extract_variable_name(message)
}

// Helper: Get code snippet at a specific range
fn get_code_at_range(code: &str, range: &Range) -> String {
    let lines: Vec<&str> = code.lines().collect();
    let start_line = range.start.line as usize;
    let end_line = range.end.line as usize;
    if start_line >= lines.len() {
        return String::new();
    }
    
    if start_line == end_line {
        let line = lines[start_line];
        let start_char = range.start.character as usize;
        let end_char = range.end.character as usize;
        
        let chars: Vec<char> = line.chars().collect();
        let s = std::cmp::min(start_char, chars.len());
        let e = std::cmp::min(end_char, chars.len());
        if s <= e {
            return chars[s..e].iter().collect();
        }
    }
    
    String::new()
}

// Helper: Locate where a variable was defined via `let x` or `let mut x` before current line
fn find_let_binding_range(code: &str, var_name: &str, current_line: u32) -> Option<Range> {
    let lines: Vec<&str> = code.lines().collect();
    let limit = std::cmp::min(current_line as usize, lines.len());
    
    for i in (0..limit).rev() {
        let line = lines[i];
        let trimmed = line.trim_start();
        if trimmed.starts_with("let ") {
            // Check if variable name is in it
            if let Some(pos) = line.find(&format!("let {var_name}")) {
                let start_char = pos as u32;
                let end_char = (pos + 4 + var_name.len()) as u32; // "let " is 4 chars
                return Some(Range {
                    start: Position { line: i as u32, character: start_char },
                    end: Position { line: i as u32, character: end_char },
                });
            }
        }
    }
    None
}
