use crate::utils::resolve_project_dir;
use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::process::Command;

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
                status: status,
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

pub async fn git_stage_all(Json(payload): Json<GitRequest>) -> Json<GitCommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    match run_git(&dir, &["add", "-A"]) {
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

pub async fn git_unstage_all(Json(payload): Json<GitRequest>) -> Json<GitCommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    match run_git(&dir, &["reset", "HEAD", "--", "."]) {
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

pub async fn git_discard_all(Json(payload): Json<GitRequest>) -> Json<GitCommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);

    // First, run checkout to discard tracked changes
    if let Err(e) = run_git(&dir, &["checkout", "--", "."]) {
        return Json(GitCommandResponse {
            success: false,
            output: String::new(),
            error: Some(format!("Failed to checkout tracked changes: {}", e)),
        });
    }

    // Second, run clean to discard untracked files
    match run_git(&dir, &["clean", "-fd"]) {
        Ok(out) => Json(GitCommandResponse {
            success: true,
            output: out,
            error: None,
        }),
        Err(e) => Json(GitCommandResponse {
            success: false,
            output: String::new(),
            error: Some(format!("Failed to clean untracked files: {}", e)),
        }),
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
        let len = clean[pos + 1..].parse::<usize>().unwrap_or(1);
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
        match run_git(
            &dir,
            &["diff", "--no-index", "/dev/null", &payload.file_path],
        ) {
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
    let virtual_project_path = payload
        .project_path
        .clone()
        .unwrap_or_else(|| format!("/Codedroid_Projects/{}", payload.project_name));
    let target_dir = resolve_project_dir(&virtual_project_path);

    // Get the parent folder
    let parent_dir = match std::path::Path::new(&target_dir).parent() {
        Some(p) => p.to_string_lossy().into_owned(),
        None => {
            return Json(GitCommandResponse {
                success: false,
                output: String::new(),
                error: Some("Could not resolve parent directory for project".to_string()),
            })
        }
    };

    // Make sure parent dir exists
    let _ = std::fs::create_dir_all(&parent_dir);

    // Run git clone clone_url project_name inside parent_dir
    match run_git(
        &parent_dir,
        &["clone", &payload.clone_url, &payload.project_name],
    ) {
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

    match run_git(
        &dir,
        &["log", "--pretty=format:%h|%s|%d|%an|%ar", "-n", "30"],
    ) {
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

        let main_items: Vec<String> = modified_files
            .iter()
            .chain(added_files.iter())
            .map(|(_, f)| f.clone())
            .take(3)
            .collect();

        if !main_items.is_empty() {
            details = main_items.join(", ");
        }

        let has_rust = modified_files
            .iter()
            .chain(added_files.iter())
            .any(|(_, f)| f.ends_with(".rs"));
        let has_css = modified_files
            .iter()
            .chain(added_files.iter())
            .any(|(_, f)| f.ends_with(".css"));

        if has_css {
            suggestions.push("style: refine UI aesthetics and stylesheet layout".to_string());
            suggestions.push("style(ui): polish sidebar design and colors".to_string());
        }

        if !added_files.is_empty() {
            let added_names: Vec<String> =
                added_files.iter().map(|(_, f)| f.clone()).take(2).collect();
            suggestions.push(format!(
                "feat: implement new files ({})",
                added_names.join(", ")
            ));
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

        if modified_files
            .iter()
            .chain(added_files.iter())
            .any(|(p, _)| p.contains("git_panel") || p.contains("git.rs"))
        {
            suggestions.push("feat: implement git integration in backend and add auto-language detection to run operations".to_string());
            suggestions.push(
                "refactor: update git panel layout with premium VS Code-style UX".to_string(),
            );
        }
    }

    if suggestions.is_empty() {
        suggestions.push("chore: update project source files".to_string());
        suggestions.push("refactor: general code improvements".to_string());
    }

    suggestions.dedup();
    Ok(suggestions)
}

pub async fn git_ai_commit_message(
    Json(payload): Json<GitRequest>,
) -> Json<GitCommitMessageResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    match generate_ai_commit_message(&dir) {
        Ok(suggestions) => {
            let default_msg = suggestions
                .first()
                .cloned()
                .unwrap_or_else(|| "style: refine codebase".to_string());
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

#[derive(Deserialize)]
pub struct GitBranchRequest {
    pub project_path: String,
    pub branch_name: String,
    pub start_point: Option<String>,
}

#[derive(Deserialize)]
pub struct GitMergeRequest {
    pub project_path: String,
    pub branch_name: String,
}

#[derive(Deserialize)]
pub struct GitRemoteRequest {
    pub project_path: String,
    pub remote_name: String,
    pub remote_url: Option<String>,
}

#[derive(Serialize)]
pub struct GitBranchesResponse {
    pub current: String,
    pub local: Vec<String>,
    pub remote: Vec<String>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct GitRemoteInfo {
    pub name: String,
    pub url: String,
}

#[derive(Serialize)]
pub struct GitRemotesResponse {
    pub remotes: Vec<GitRemoteInfo>,
    pub error: Option<String>,
}

pub async fn git_list_branches(Json(payload): Json<GitRequest>) -> Json<GitBranchesResponse> {
    let dir = resolve_project_dir(&payload.project_path);

    if let Err(e) = run_git(&dir, &["status"]) {
        return Json(GitBranchesResponse {
            current: "None".to_string(),
            local: vec![],
            remote: vec![],
            error: Some(format!("Not a git repository: {}", e)),
        });
    }

    let current = match run_git(&dir, &["rev-parse", "--abbrev-ref", "HEAD"]) {
        Ok(out) => out.trim().to_string(),
        Err(_) => "HEAD".to_string(),
    };

    let local = match run_git(&dir, &["branch"]) {
        Ok(out) => {
            let mut list = Vec::new();
            for line in out.lines() {
                let name = line.trim().trim_start_matches('*').trim().to_string();
                if !name.is_empty() {
                    list.push(name);
                }
            }
            list
        }
        Err(_) => vec![],
    };

    let remote = match run_git(&dir, &["branch", "-r"]) {
        Ok(out) => {
            let mut list = Vec::new();
            for line in out.lines() {
                let name = line.trim().to_string();
                if !name.is_empty() && !name.contains("->") {
                    list.push(name);
                }
            }
            list
        }
        Err(_) => vec![],
    };

    Json(GitBranchesResponse {
        current,
        local,
        remote,
        error: None,
    })
}

pub async fn git_create_branch(Json(payload): Json<GitBranchRequest>) -> Json<GitCommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    let mut args = vec!["checkout", "-b", &payload.branch_name];
    if let Some(ref start) = payload.start_point {
        if !start.trim().is_empty() {
            args.push(start);
        }
    }
    match run_git(&dir, &args) {
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

pub async fn git_checkout_branch(
    Json(payload): Json<GitBranchRequest>,
) -> Json<GitCommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    let branch_to_checkout = payload.branch_name.clone();
    let mut args = vec!["checkout"];
    if branch_to_checkout.starts_with("origin/") {
        let local_name = branch_to_checkout.strip_prefix("origin/").unwrap();
        let local_exists = match run_git(
            &dir,
            &[
                "show-ref",
                "--verify",
                &format!("refs/heads/{}", local_name),
            ],
        ) {
            Ok(_) => true,
            Err(_) => false,
        };
        if local_exists {
            args.push(local_name);
        } else {
            args.push("-b");
            args.push(local_name);
            args.push("--track");
            args.push(&payload.branch_name);
        }
    } else {
        args.push(&branch_to_checkout);
    }

    match run_git(&dir, &args) {
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

pub async fn git_merge_branch(Json(payload): Json<GitMergeRequest>) -> Json<GitCommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    match run_git(&dir, &["merge", &payload.branch_name]) {
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

pub async fn git_list_remotes(Json(payload): Json<GitRequest>) -> Json<GitRemotesResponse> {
    let dir = resolve_project_dir(&payload.project_path);

    if let Err(e) = run_git(&dir, &["status"]) {
        return Json(GitRemotesResponse {
            remotes: vec![],
            error: Some(format!("Not a git repository: {}", e)),
        });
    }

    match run_git(&dir, &["remote", "-v"]) {
        Ok(out) => {
            let mut remotes = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for line in out.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[0].to_string();
                    let url = parts[1].to_string();
                    if seen.insert(name.clone()) {
                        remotes.push(GitRemoteInfo { name, url });
                    }
                }
            }
            Json(GitRemotesResponse {
                remotes,
                error: None,
            })
        }
        Err(e) => Json(GitRemotesResponse {
            remotes: vec![],
            error: Some(e),
        }),
    }
}

pub async fn git_add_remote(Json(payload): Json<GitRemoteRequest>) -> Json<GitCommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    let url = match payload.remote_url {
        Some(u) => u,
        None => {
            return Json(GitCommandResponse {
                success: false,
                output: String::new(),
                error: Some("Remote URL is required".to_string()),
            })
        }
    };
    match run_git(&dir, &["remote", "add", &payload.remote_name, &url]) {
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

pub async fn git_remove_remote(Json(payload): Json<GitRemoteRequest>) -> Json<GitCommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    match run_git(&dir, &["remote", "remove", &payload.remote_name]) {
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

pub async fn git_set_remote_url(Json(payload): Json<GitRemoteRequest>) -> Json<GitCommandResponse> {
    let dir = resolve_project_dir(&payload.project_path);
    let url = match payload.remote_url {
        Some(u) => u,
        None => {
            return Json(GitCommandResponse {
                success: false,
                output: String::new(),
                error: Some("Remote URL is required".to_string()),
            })
        }
    };
    match run_git(&dir, &["remote", "set-url", &payload.remote_name, &url]) {
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

pub fn router() -> Router {
    Router::new()
        .route("/status", post(git_status))
        .route("/stage", post(git_stage))
        .route("/stage-all", post(git_stage_all))
        .route("/unstage", post(git_unstage))
        .route("/unstage-all", post(git_unstage_all))
        .route("/discard", post(git_discard))
        .route("/discard-all", post(git_discard_all))
        .route("/commit", post(git_commit))
        .route("/push", post(git_push))
        .route("/pull", post(git_pull))
        .route("/diff_lines", post(git_diff_lines))
        .route("/diff_text", post(git_diff_text))
        .route("/clone", post(git_clone))
        .route("/log", post(git_log))
        .route("/ai_commit_message", post(git_ai_commit_message))
        .route("/branches", post(git_list_branches))
        .route("/branch/create", post(git_create_branch))
        .route("/branch/checkout", post(git_checkout_branch))
        .route("/branch/merge", post(git_merge_branch))
        .route("/remotes", post(git_list_remotes))
        .route("/remote/add", post(git_add_remote))
        .route("/remote/remove", post(git_remove_remote))
        .route("/remote/set-url", post(git_set_remote_url))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::process::Command;

    fn init_git_repo(path: &str) {
        let _ = fs::remove_dir_all(path);
        fs::create_dir_all(path).unwrap();

        Command::new("git")
            .arg("init")
            .current_dir(path)
            .output()
            .expect("Failed to init git repo");

        // Configure local git user for test
        Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(path)
            .output()
            .unwrap();
        Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .current_dir(path)
            .output()
            .unwrap();
    }

    #[tokio::test]
    async fn test_git_stage_unstage_discard_all() {
        let project_path = "./temp_git_test".to_string();
        init_git_repo(&project_path);

        // 1. Create a dummy file
        let file_path = format!("{}/test.txt", project_path);
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Initial content").unwrap();

        // Commit it initially so we can test discarding modification
        Command::new("git")
            .args(&["add", "test.txt"])
            .current_dir(&project_path)
            .output()
            .unwrap();
        Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(&project_path)
            .output()
            .unwrap();

        // 2. Modify the file and create a new untracked file
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Modified content").unwrap();

        let untracked_path = format!("{}/untracked.txt", project_path);
        let mut untracked_file = File::create(&untracked_path).unwrap();
        writeln!(untracked_file, "Untracked file").unwrap();

        // Verify they are unstaged
        let status_res = git_status(axum::Json(GitRequest {
            project_path: project_path.clone(),
        }))
        .await;

        let staged_count = |res: &GitStatusResponse| {
            res.files
                .iter()
                .filter(|f| {
                    let s = &f.status;
                    s != "??"
                        && s.chars()
                            .next()
                            .map(|c| c != ' ' && c != '?')
                            .unwrap_or(false)
                })
                .count()
        };
        let unstaged_count = |res: &GitStatusResponse| {
            res.files
                .iter()
                .filter(|f| {
                    let s = &f.status;
                    s == "??" || s.chars().nth(1).map(|c| c != ' ').unwrap_or(false)
                })
                .count()
        };

        assert!(
            unstaged_count(&status_res.0) > 0,
            "Should have unstaged changes"
        );

        // 3. Stage all
        let stage_res = git_stage_all(axum::Json(GitRequest {
            project_path: project_path.clone(),
        }))
        .await;
        assert!(stage_res.0.success, "Stage all should succeed");

        let status_res2 = git_status(axum::Json(GitRequest {
            project_path: project_path.clone(),
        }))
        .await;
        assert_eq!(
            unstaged_count(&status_res2.0),
            0,
            "Should have no unstaged changes after stage-all"
        );
        assert!(
            staged_count(&status_res2.0) > 0,
            "Should have staged changes after stage-all"
        );

        // 4. Unstage all
        let unstage_res = git_unstage_all(axum::Json(GitRequest {
            project_path: project_path.clone(),
        }))
        .await;
        assert!(unstage_res.0.success, "Unstage all should succeed");

        let status_res3 = git_status(axum::Json(GitRequest {
            project_path: project_path.clone(),
        }))
        .await;
        for f in &status_res3.0.files {
            eprintln!("DEBUG: path={}, status='{}'", f.path, f.status);
        }
        assert!(
            unstaged_count(&status_res3.0) > 0,
            "Should have unstaged changes after unstage-all"
        );
        assert_eq!(
            staged_count(&status_res3.0),
            0,
            "Should have no staged changes after unstage-all"
        );

        // 5. Discard all
        let discard_res = git_discard_all(axum::Json(GitRequest {
            project_path: project_path.clone(),
        }))
        .await;
        assert!(discard_res.0.success, "Discard all should succeed");

        let status_res4 = git_status(axum::Json(GitRequest {
            project_path: project_path.clone(),
        }))
        .await;
        assert_eq!(
            unstaged_count(&status_res4.0),
            0,
            "Should have no unstaged changes after discard-all"
        );
        assert_eq!(
            staged_count(&status_res4.0),
            0,
            "Should have no staged changes after discard-all"
        );

        // Verify files were reset/cleaned
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content.trim(), "Initial content");
        assert!(
            !std::path::Path::new(&untracked_path).exists(),
            "Untracked file should be deleted"
        );

        // Cleanup
        let _ = fs::remove_dir_all(&project_path);
    }
}
