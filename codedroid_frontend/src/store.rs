use crate::models::{Project, Settings, Snippet};
use gloo_storage::{LocalStorage, Storage};
/// LocalStorage-backed reactive store (mirrors Flutter Hive+Riverpod)
use leptos::prelude::*;

// ── Projects ──────────────────────────────────────────────────────────────
pub fn load_projects() -> Vec<Project> {
    LocalStorage::get("codedroid_projects").unwrap_or_default()
}

pub fn save_projects(projects: &[Project]) {
    let _ = LocalStorage::set("codedroid_projects", projects);
}

pub fn add_project(projects: &RwSignal<Vec<Project>>, p: Project) {
    projects.update(|v| v.push(p));
    save_projects(&projects.get_untracked());
}

pub fn clear_project_files_from_local_storage(project_id: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let prefix = format!("codedroid_file_{}_", project_id);
            let mut keys_to_remove = Vec::new();
            let len = storage.length().unwrap_or(0);
            for i in 0..len {
                if let Ok(Some(k)) = storage.key(i) {
                    if k.starts_with(&prefix) {
                        keys_to_remove.push(k);
                    }
                }
            }
            for k in keys_to_remove {
                let _ = storage.remove_item(&k);
            }
        }
    }
}

pub fn delete_project(projects: &RwSignal<Vec<Project>>, id: &str) {
    projects.update(|v| v.retain(|p| p.id != id));
    save_projects(&projects.get_untracked());
    clear_project_files_from_local_storage(id);
    let _ = LocalStorage::delete(&format!("codedroid_term_sessions_{}", id));
    let _ = LocalStorage::delete(&format!("codedroid_term_active_idx_{}", id));
}

// ── Settings ──────────────────────────────────────────────────────────────
pub fn load_settings() -> Settings {
    LocalStorage::get("codedroid_settings").unwrap_or_default()
}

pub fn save_settings(s: &Settings) {
    let _ = LocalStorage::set("codedroid_settings", s);
}

// ── Snippets ──────────────────────────────────────────────────────────────
#[allow(dead_code)]
pub fn load_snippets() -> Vec<Snippet> {
    LocalStorage::get("codedroid_snippets").unwrap_or_default()
}

#[allow(dead_code)]
pub fn save_snippets(snippets: &[Snippet]) {
    let _ = LocalStorage::set("codedroid_snippets", snippets);
}

// ── Virtual Files (code per project/file) ─────────────────────────────────
/// Marker stored in localStorage when a file is indexed but not yet loaded from disk.
/// Must never be written back to the host filesystem (see sync_project / save_current).
pub const UNLOADED_PLACEHOLDER: &str = "__CODEDROID_NOT_LOADED__";

pub fn is_unloaded_placeholder(content: &str) -> bool {
    content == UNLOADED_PLACEHOLDER
}

/// True when editor cache has no real file body yet (empty or unload marker).
pub fn needs_load_from_disk(content: &str) -> bool {
    content.is_empty() || is_unloaded_placeholder(content)
}

/// True when this cache entry must not be pushed to disk (prevents wiping source files).
pub fn should_skip_disk_sync(content: &str) -> bool {
    needs_load_from_disk(content)
}

pub fn load_file(key: &str) -> String {
    LocalStorage::get::<String>(key).unwrap_or_default()
}

pub fn save_file(key: &str, content: &str) {
    let _ = LocalStorage::set(key, content);
}

pub fn file_key(project_id: &str, filename: &str) -> String {
    format!("codedroid_file_{}_{}", project_id, filename)
}

// ── Editor sidebar (explorer / search / git) ─────────────────────────────
pub fn load_sidebar_open(project_id: &str) -> bool {
    LocalStorage::get(&format!("codedroid_sidebar_open_{}", project_id)).unwrap_or(true)
}

pub fn save_sidebar_open(project_id: &str, open: bool) {
    let _ = LocalStorage::set(&format!("codedroid_sidebar_open_{}", project_id), &open);
}

pub fn load_sidebar_mode(project_id: &str) -> usize {
    LocalStorage::get(&format!("codedroid_sidebar_mode_{}", project_id)).unwrap_or(0)
}

pub fn save_sidebar_mode(project_id: &str, mode: usize) {
    let _ = LocalStorage::set(&format!("codedroid_sidebar_mode_{}", project_id), &mode);
}

pub fn load_chat_open(project_id: &str) -> bool {
    LocalStorage::get(&format!("codedroid_chat_open_{}", project_id)).unwrap_or(false)
}

pub fn save_chat_open(project_id: &str, open: bool) {
    let _ = LocalStorage::set(&format!("codedroid_chat_open_{}", project_id), &open);
}

pub fn load_bottom_open(project_id: &str) -> bool {
    LocalStorage::get(&format!("codedroid_bottom_open_{}", project_id)).unwrap_or(true)
}

pub fn save_bottom_open(project_id: &str, open: bool) {
    let _ = LocalStorage::set(&format!("codedroid_bottom_open_{}", project_id), &open);
}

// ── Terminal Command History ──────────────────────────────────────────────
pub fn load_terminal_history(project_id: &str) -> Vec<String> {
    LocalStorage::get(&format!("codedroid_term_history_{}", project_id)).unwrap_or_default()
}

pub fn save_terminal_history(project_id: &str, history: &[String]) {
    let _ = LocalStorage::set(&format!("codedroid_term_history_{}", project_id), history);
}

// ── Side Panel Sizes ──────────────────────────────────────────────────────
pub fn load_sidebar_width(project_id: &str) -> i32 {
    LocalStorage::get(&format!("codedroid_sidebar_width_{}", project_id)).unwrap_or(240)
}

#[allow(dead_code)]
pub fn save_sidebar_width(project_id: &str, width: i32) {
    let _ = LocalStorage::set(&format!("codedroid_sidebar_width_{}", project_id), &width);
}

pub fn load_agent_width(project_id: &str) -> i32 {
    LocalStorage::get(&format!("codedroid_agent_width_{}", project_id)).unwrap_or(360)
}

#[allow(dead_code)]
pub fn save_agent_width(project_id: &str, width: i32) {
    let _ = LocalStorage::set(&format!("codedroid_agent_width_{}", project_id), &width);
}

pub fn load_bottom_height(project_id: &str) -> i32 {
    LocalStorage::get(&format!("codedroid_bottom_height_{}", project_id)).unwrap_or(240)
}

#[allow(dead_code)]
pub fn save_bottom_height(project_id: &str, height: i32) {
    let _ = LocalStorage::set(&format!("codedroid_bottom_height_{}", project_id), &height);
}

pub fn load_preview_width(project_id: &str) -> i32 {
    LocalStorage::get(&format!("codedroid_preview_width_{}", project_id)).unwrap_or(360)
}

#[allow(dead_code)]
pub fn save_preview_width(project_id: &str, width: i32) {
    let _ = LocalStorage::set(&format!("codedroid_preview_width_{}", project_id), &width);
}

