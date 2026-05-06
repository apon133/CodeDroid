/// LocalStorage-backed reactive store (mirrors Flutter Hive+Riverpod)
use leptos::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use crate::models::{Project, Settings, Snippet};

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

pub fn delete_project(projects: &RwSignal<Vec<Project>>, id: &str) {
    projects.update(|v| v.retain(|p| p.id != id));
    save_projects(&projects.get_untracked());
}

// ── Settings ──────────────────────────────────────────────────────────────
pub fn load_settings() -> Settings {
    LocalStorage::get("codedroid_settings").unwrap_or_default()
}

pub fn save_settings(s: &Settings) {
    let _ = LocalStorage::set("codedroid_settings", s);
}

// ── Snippets ──────────────────────────────────────────────────────────────
pub fn load_snippets() -> Vec<Snippet> {
    LocalStorage::get("codedroid_snippets").unwrap_or_default()
}

pub fn save_snippets(snippets: &[Snippet]) {
    let _ = LocalStorage::set("codedroid_snippets", snippets);
}

// ── Virtual Files (code per project/file) ─────────────────────────────────
pub fn load_file(key: &str) -> String {
    LocalStorage::get::<String>(key).unwrap_or_default()
}

pub fn save_file(key: &str, content: &str) {
    let _ = LocalStorage::set(key, content);
}

pub fn file_key(project_id: &str, filename: &str) -> String {
    format!("codedroid_file_{}_{}", project_id, filename)
}
