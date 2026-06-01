use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::process::Command;
use crate::utils::resolve_project_dir;

#[derive(Deserialize)]
pub struct GitRequest {
    pub project_path: String,
}

#[derive(Deserialize)]
pub struct GitFileRequest {
    pub project_path: String,
    pub file_path: String,
}

#[derive(Deserialize)]
pub struct GitCommitRequest {
    pub project_path: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct GitCloneRequest {
    pub clone_url: String,
    pub project_name: String,
    pub project_path: Option<String>,
}

#[derive(Serialize)]
pub struct GitStatusResponse {
    pub branch: String,
    pub files: Vec<GitFileStatus>,
    pub error: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct GitFileStatus {
    pub path: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct GitCommandResponse {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct GitDiffLinesResponse {
    pub added: Vec<usize>,
    pub modified: Vec<usize>,
    pub deleted: Vec<usize>,
}

fn run_git(dir: &str, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(dir);

    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            if output.status.success() {
                Ok(stdout)
            } else {
                Err(if stderr.is_empty() { stdout } else { stderr })
            }
        }
        Err(e) => Err(format!("Failed to run git command: {}", e)),
    }
}

pub async fn git_status(Json(payload): Json<GitRequest>) -> Json<GitStatusResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    
    // Check if git is initialized
    if let Err(e) = run_git(&dir, &["status"]) {
        return Json(GitStatusResponse {
            branch: "None".to_string(),
            files: vec![],
            error: Some(format!("Not a git repository: {}", e)),
        });
    }

    let branch = run_git(&dir, &["rev-parse", "--abbrev-ref", "HEAD"])
        .unwrap_or_else(|_| "HEAD".to_string())
        .trim()
        .to_string();

    let porcelain = match run_git(&dir, &["status", "--porcelain"]) {
        Ok(out) => out,
        Err(e) => {
            return Json(GitStatusResponse {
                branch,
                files: vec![],
                error: Some(e),
            });
        }
    };

    let mut files = Vec::new();
    for line in porcelain.lines() {
        if line.len() >= 4 {
            let status = line[..2].to_string();
            let file_path = if status.starts_with('R') {
                if let Some(pos) = line[3..].find(" -> ") {
                    line[3 + pos + 4..].trim_matches('"').to_string()
                } else {
                    line[3..].trim_matches('"').to_string()
                }
            } else {
                line[3..].trim_matches('"').to_string()
            };
            files.push(GitFileStatus {
                path: file_path,
                status: status.trim().to_string(),
            });
        }
    }

    Json(GitStatusResponse {
        branch,
        files,
        error: None,
    })
}

pub async fn git_stage(Json(payload): Json<GitFileRequest>) -> Json<GitCommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    match run_git(&dir, &["add", &payload.file_path]) {
        Ok(out) => Json(GitCommandResponse {
            success: true,
            output: out,
            error: None,
        }),
        Err(e) => Json(GitCommandResponse {
            success: false,
            output: String::new(),
            error: Some(e),
        }),
    }
}

pub async fn git_unstage(Json(payload): Json<GitFileRequest>) -> Json<GitCommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    match run_git(&dir, &["reset", "HEAD", "--", &payload.file_path]) {
        Ok(out) => Json(GitCommandResponse {
            success: true,
            output: out,
            error: None,
        }),
        Err(e) => Json(GitCommandResponse {
            success: false,
            output: String::new(),
            error: Some(e),
        }),
    }
}

pub async fn git_discard(Json(payload): Json<GitFileRequest>) -> Json<GitCommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    
    // First try checking if the file is tracked
    let is_tracked = match run_git(&dir, &["ls-files", "--error-unmatch", &payload.file_path]) {
        Ok(_) => true,
        Err(_) => false,
    };

    if is_tracked {
        // Discard tracked changes
        match run_git(&dir, &["checkout", "--", &payload.file_path]) {
            Ok(out) => Json(GitCommandResponse {
                success: true,
                output: out,
                error: None,
            }),
            Err(e) => Json(GitCommandResponse {
                success: false,
                output: String::new(),
                error: Some(e),
            }),
        }
    } else {
        // Untracked file: remove it
        let file_path_obj = std::path::Path::new(&dir).join(&payload.file_path);
        if file_path_obj.exists() {
            if file_path_obj.is_dir() {
                match std::fs::remove_dir_all(&file_path_obj) {
                    Ok(_) => Json(GitCommandResponse {
                        success: true,
                        output: "Directory deleted".to_string(),
                        error: None,
                    }),
                    Err(e) => Json(GitCommandResponse {
                        success: false,
                        output: String::new(),
                        error: Some(e.to_string()),
                    }),
                }
            } else {
                match std::fs::remove_file(&file_path_obj) {
                    Ok(_) => Json(GitCommandResponse {
                        success: true,
                        output: "File deleted".to_string(),
                        error: None,
                    }),
                    Err(e) => Json(GitCommandResponse {
                        success: false,
                        output: String::new(),
                        error: Some(e.to_string()),
                    }),
                }
            }
        } else {
            Json(GitCommandResponse {
                success: true,
                output: "File did not exist".to_string(),
                error: None,
            })
        }
    }
}

pub async fn git_commit(Json(payload): Json<GitCommitRequest>) -> Json<GitCommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    match run_git(&dir, &["commit", "-m", &payload.message]) {
        Ok(out) => Json(GitCommandResponse {
            success: true,
            output: out,
            error: None,
        }),
        Err(e) => Json(GitCommandResponse {
            success: false,
            output: String::new(),
            error: Some(e),
        }),
    }
}

pub async fn git_push(Json(payload): Json<GitRequest>) -> Json<GitCommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    match run_git(&dir, &["push"]) {
        Ok(out) => Json(GitCommandResponse {
            success: true,
            output: out,
            error: None,
        }),
        Err(e) => Json(GitCommandResponse {
            success: false,
            output: String::new(),
            error: Some(e),
        }),
    }
}

pub async fn git_pull(Json(payload): Json<GitRequest>) -> Json<GitCommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    match run_git(&dir, &["pull"]) {
        Ok(out) => Json(GitCommandResponse {
            success: true,
            output: out,
            error: None,
        }),
        Err(e) => Json(GitCommandResponse {
            success: false,
            output: String::new(),
            error: Some(e),
        }),
    }
}

pub async fn git_diff_lines(Json(payload): Json<GitFileRequest>) -> Json<GitDiffLinesResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut deleted = Vec::new();

    // Run git diff -U0 for the file
    if let Ok(diff_out) = run_git(&dir, &["diff", "-U0", "--", &payload.file_path]) {
        for line in diff_out.lines() {
            if line.starts_with("@@") {
                // Parse @@ -old_start,old_len +new_start,new_len @@
                // Examples:
                // @@ -5 +5,2 @@
                // @@ -10,3 +10,3 @@
                // @@ -10,3 +10,0 @@
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let old_part = parts[1]; // -10,3
                    let new_part = parts[2]; // +10,3

                    let (_old_start, old_len) = parse_diff_part(old_part);
                    let (new_start, new_len) = parse_diff_part(new_part);

                    if old_len == 0 {
                        // Purely added lines
                        for l in new_start..(new_start + new_len) {
                            added.push(l);
                        }
                    } else if new_len == 0 {
                        // Purely deleted lines
                        // In new file, deletion is marked at new_start
                        deleted.push(new_start);
                    } else {
                        // Modified/replaced lines
                        // Mark the lines in the new file as modified
                        for l in new_start..(new_start + new_len) {
                            modified.push(l);
                        }
                    }
                }
            }
        }
    }

    Json(GitDiffLinesResponse {
        added,
        modified,
        deleted,
    })
}

// Parses "+10,3" into (10, 3) or "-5" into (5, 1)
fn parse_diff_part(part: &str) -> (usize, usize) {
    let clean = part.trim_start_matches(|c| c == '+' || c == '-');
    if let Some(pos) = clean.find(',') {
        let start = clean[..pos].parse::<usize>().unwrap_or(1);
        let len = clean[pos+1..].parse::<usize>().unwrap_or(1);
        (start, len)
    } else {
        let start = clean.parse::<usize>().unwrap_or(1);
        (start, 1)
    }
}

pub async fn git_diff_text(Json(payload): Json<GitFileRequest>) -> Json<GitCommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    
    // Check if the file is tracked
    let is_tracked = match run_git(&dir, &["ls-files", "--error-unmatch", &payload.file_path]) {
        Ok(_) => true,
        Err(_) => false,
    };

    if is_tracked {
        match run_git(&dir, &["diff", "HEAD", "--", &payload.file_path]) {
            Ok(out) => Json(GitCommandResponse {
                success: true,
                output: out,
                error: None,
            }),
            Err(e) => Json(GitCommandResponse {
                success: false,
                output: String::new(),
                error: Some(e),
            }),
        }
    } else {
        // For untracked files, git diff HEAD doesn't show anything unless we use a special trick.
        // We can just run git diff --no-index /dev/null <file_path>
        // But since we are inside `dir`, the path to file is relative `payload.file_path`
        match run_git(&dir, &["diff", "--no-index", "/dev/null", &payload.file_path]) {
            Ok(out) => Json(GitCommandResponse {
                success: true,
                output: out,
                error: None,
            }),
            Err(e) => {
                // git diff --no-index exits with 1 if diff found, which makes run_git return Err(stdout)!
                // So if stdout is not empty and starts with diff, it is a success.
                if e.starts_with("diff") {
                    Json(GitCommandResponse {
                        success: true,
                        output: e,
                        error: None,
                    })
                } else {
                    Json(GitCommandResponse {
                        success: false,
                        output: String::new(),
                        error: Some(e),
                    })
                }
            }
        }
    }
}

pub async fn git_clone(Json(payload): Json<GitCloneRequest>) -> Json<GitCommandResponse> {
    // Determine target directory
    let virtual_project_path = payload.project_path.clone().unwrap_or_else(|| {
        format!("/Codedroid_Projects/{}", payload.project_name)
    });
    let target_dir = resolve_project_dir(&virtual_project_path);

    // Get the parent folder
    let parent_dir = match std::path::Path::new(&target_dir).parent() {
        Some(p) => p.to_string_lossy().into_owned(),
        None => return Json(GitCommandResponse {
            success: false,
            output: String::new(),
            error: Some("Could not resolve parent directory for project".to_string()),
        }),
    };

    // Make sure parent dir exists
    let _ = std::fs::create_dir_all(&parent_dir);

    // Run git clone clone_url project_name inside parent_dir
    match run_git(&parent_dir, &["clone", &payload.clone_url, &payload.project_name]) {
        Ok(out) => Json(GitCommandResponse {
            success: true,
            output: out,
            error: None,
        }),
        Err(e) => Json(GitCommandResponse {
            success: false,
            output: String::new(),
            error: Some(e),
        }),
    }
}

#[derive(Serialize)]
pub struct GitCommitInfo {
    pub hash: String,
    pub subject: String,
    pub refs: String,
    pub author_name: String,
    pub relative_date: String,
}

#[derive(Serialize)]
pub struct GitLogResponse {
    pub commits: Vec<GitCommitInfo>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct GitCommitMessageResponse {
    pub message: String,
    pub suggestions: Vec<String>,
    pub error: Option<String>,
}

pub async fn git_log(Json(payload): Json<GitRequest>) -> Json<GitLogResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    
    // Check if git is initialized
    if let Err(e) = run_git(&dir, &["status"]) {
        return Json(GitLogResponse {
            commits: vec![],
            error: Some(format!("Not a git repository: {}", e)),
        });
    }

    match run_git(&dir, &["log", "--pretty=format:%h|%s|%d|%an|%ar", "-n", "30"]) {
        Ok(out) => {
            let mut commits = Vec::new();
            for line in out.lines() {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 5 {
                    commits.push(GitCommitInfo {
                        hash: parts[0].to_string(),
                        subject: parts[1].to_string(),
                        refs: parts[2].trim().to_string(),
                        author_name: parts[3].to_string(),
                        relative_date: parts[4].to_string(),
                    });
                } else if !line.trim().is_empty() {
                    commits.push(GitCommitInfo {
                        hash: parts.first().copied().unwrap_or("").to_string(),
                        subject: parts.get(1).copied().unwrap_or("").to_string(),
                        refs: String::new(),
                        author_name: String::new(),
                        relative_date: String::new(),
                    });
                }
            }
            Json(GitLogResponse {
                commits,
                error: None,
            })
        }
        Err(e) => Json(GitLogResponse {
            commits: vec![],
            error: Some(e),
        }),
    }
}

fn generate_ai_commit_message(dir: &str) -> Result<Vec<String>, String> {
    // 1. Get staged changes first
    let mut diff = match run_git(dir, &["diff", "--cached", "--name-status"]) {
        Ok(d) => d,
        Err(_) => String::new(),
    };
    
    if diff.trim().is_empty() {
        // Fallback to unstaged changes
        diff = match run_git(dir, &["diff", "--name-status"]) {
            Ok(d) => d,
            Err(_) => String::new(),
        };
    }

    if diff.trim().is_empty() {
        return Ok(vec![
            "style: minor updates to files".to_string(),
            "docs: update documentation".to_string(),
            "chore: general maintenance updates".to_string(),
        ]);
    }

    let mut added_files = Vec::new();
    let mut modified_files = Vec::new();
    let mut deleted_files = Vec::new();

    for line in diff.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let status = parts[0];
            let filepath = parts[1];
            let filename = std::path::Path::new(filepath)
                .file_name()
                .map(|f| f.to_string_lossy().into_owned())
                .unwrap_or_else(|| filepath.to_string());

            if status.starts_with('A') {
                added_files.push((filepath.to_string(), filename));
            } else if status.starts_with('D') {
                deleted_files.push((filepath.to_string(), filename));
            } else {
                modified_files.push((filepath.to_string(), filename));
            }
        }
    }

    let mut suggestions = Vec::new();

    if !modified_files.is_empty() || !added_files.is_empty() {
        let mut scope = "";
        let mut details = String::new();

        if modified_files.iter().any(|(p, _)| p.contains("api/")) {
            scope = "api";
        } else if modified_files.iter().any(|(p, _)| p.contains("frontend/")) {
            scope = "ui";
        }

        let main_items: Vec<String> = modified_files.iter()
            .chain(added_files.iter())
            .map(|(_, f)| f.clone())
            .take(3)
            .collect();

        if !main_items.is_empty() {
            details = main_items.join(", ");
        }

        let has_rust = modified_files.iter().chain(added_files.iter()).any(|(_, f)| f.ends_with(".rs"));
        let has_css = modified_files.iter().chain(added_files.iter()).any(|(_, f)| f.ends_with(".css"));

        if has_css {
            suggestions.push("style: refine UI aesthetics and stylesheet layout".to_string());
            suggestions.push("style(ui): polish sidebar design and colors".to_string());
        }

        if !added_files.is_empty() {
            let added_names: Vec<String> = added_files.iter().map(|(_, f)| f.clone()).take(2).collect();
            suggestions.push(format!("feat: implement new files ({})", added_names.join(", ")));
            if scope == "ui" {
                suggestions.push("feat(ui): add new components and views".to_string());
            } else {
                suggestions.push("feat: integrate new modules and services".to_string());
            }
        }

        if !modified_files.is_empty() {
            if has_rust {
                suggestions.push("refactor: optimize and clean up Rust implementation".to_string());
                suggestions.push("fix: resolve issues and errors in compiler flow".to_string());
            }
            suggestions.push(format!("refactor: update and modularize {}", details));
        }

        if modified_files.iter().chain(added_files.iter()).any(|(p, _)| p.contains("git_panel") || p.contains("git.rs")) {
            suggestions.push("feat: implement git integration in backend and add auto-language detection to run operations".to_string());
            suggestions.push("refactor: update git panel layout with premium VS Code-style UX".to_string());
        }
    }

    if suggestions.is_empty() {
        suggestions.push("chore: update project source files".to_string());
        suggestions.push("refactor: general code improvements".to_string());
    }

    suggestions.dedup();
    Ok(suggestions)
}

pub async fn git_ai_commit_message(Json(payload): Json<GitRequest>) -> Json<GitCommitMessageResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    match generate_ai_commit_message(&dir) {
        Ok(suggestions) => {
            let default_msg = suggestions.first().cloned().unwrap_or_else(|| "style: refine codebase".to_string());
            Json(GitCommitMessageResponse {
                message: default_msg,
                suggestions,
                error: None,
            })
        }
        Err(e) => Json(GitCommitMessageResponse {
            message: String::new(),
            suggestions: vec![],
            error: Some(e),
        }),
    }
}

pub fn router() -> Router {
    Router::new()
        .route("/status", post(git_status))
        .route("/stage", post(git_stage))
        .route("/unstage", post(git_unstage))
        .route("/discard", post(git_discard))
        .route("/commit", post(git_commit))
        .route("/push", post(git_push))
        .route("/pull", post(git_pull))
        .route("/diff_lines", post(git_diff_lines))
        .route("/diff_text", post(git_diff_text))
        .route("/clone", post(git_clone))
        .route("/log", post(git_log))
        .route("/ai_commit_message", post(git_ai_commit_message))
}
