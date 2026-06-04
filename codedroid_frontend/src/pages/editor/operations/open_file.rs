use crate::api;
use crate::pages::editor::utils::{is_absolute_path, is_media_file};
use crate::store;
use gloo_storage::Storage;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub fn make_open_file(
    pid: String,
    ppath: String,
    code: RwSignal<String>,
    active_tab: RwSignal<Option<String>>,
    open_tabs: RwSignal<Vec<String>>,
    dirty: RwSignal<bool>,
    trigger_diagnostics: Callback<String>,
) -> Callback<String> {
    Callback::new(move |name: String| {
        if is_media_file(&name) {
            open_tabs.update(|t| {
                if !t.contains(&name) {
                    t.push(name.clone());
                }
            });
            active_tab.set(Some(name.clone()));
            code.set(String::new());
            dirty.set(false);
            return;
        }
        if name.starts_with("agent-diff://") {
            open_tabs.update(|t| {
                if !t.contains(&name) {
                    t.push(name.clone());
                }
            });
            active_tab.set(Some(name.clone()));
            dirty.set(false);

            let file_path = name.strip_prefix("agent-diff://").unwrap_or(&name).to_string();
            let key = format!("agent-diff:{}:{}", pid, file_path);
            let diff_content = gloo_storage::LocalStorage::get::<String>(&key)
                .unwrap_or_else(|_| "No changes proposed for this file.".to_string());
            code.set(diff_content);
            return;
        }

        if name.starts_with("git-diff://") {
            open_tabs.update(|t| {
                if !t.contains(&name) {
                    t.push(name.clone());
                }
            });
            active_tab.set(Some(name.clone()));
            code.set("Loading diff...".to_string());
            dirty.set(false);

            let name_clone = name.clone();
            let ppath_clone = ppath.clone();
            let code_clone = code.clone();
            let active_tab_clone = active_tab.clone();

            spawn_local(async move {
                let file_path = name_clone.strip_prefix("git-diff://").unwrap_or(&name_clone).to_string();
                if let Ok(resp) = api::git_diff_text_api(&ppath_clone, &file_path).await {
                    if active_tab_clone.get_untracked().as_ref() == Some(&name_clone) {
                        code_clone.set(resp.output);
                    }
                } else {
                    if active_tab_clone.get_untracked().as_ref() == Some(&name_clone) {
                        code_clone.set("Error: Failed to fetch diff.".to_string());
                    }
                }
            });
            return;
        }

        let key = store::file_key(&pid, &name);
        let content = store::load_file(&key);
        open_tabs.update(|t| {
            if !t.contains(&name) {
                t.push(name.clone());
            }
        });
        active_tab.set(Some(name.clone()));
        code.set(content.clone());
        dirty.set(false);
        trigger_diagnostics.run(content.clone());

        let key_exists = gloo_storage::LocalStorage::get::<String>(&key).is_ok();
        let is_absolute = is_absolute_path(&name);
        if !key_exists || is_absolute || store::needs_load_from_disk(&content) {
            let name_clone = name.clone();
            let key_clone = key.clone();
            let ppath_clone = ppath.clone();
            let code_clone = code.clone();
            let active_tab_clone = active_tab.clone();
            let trigger_diag_clone = trigger_diagnostics.clone();

            spawn_local(async move {
                let file_path = if name_clone.starts_with('/') {
                    name_clone.clone()
                } else if name_clone.starts_with("Users/")
                    || name_clone.starts_with("home/")
                    || name_clone.starts_with("data/")
                {
                    format!("/{}", name_clone)
                } else if is_absolute_path(&name_clone) {
                    name_clone.clone()
                } else {
                    format!("{}/{}", ppath_clone, name_clone)
                };

                if let Ok(resp) = api::read_file_api(&file_path).await {
                    if resp.error.is_empty() {
                        store::save_file(&key_clone, &resp.content);
                        if active_tab_clone.get_untracked().as_ref() == Some(&name_clone) {
                            code_clone.set(resp.content.clone());
                            trigger_diag_clone.run(resp.content);
                        }
                    }
                }
            });
        }
    })
}
