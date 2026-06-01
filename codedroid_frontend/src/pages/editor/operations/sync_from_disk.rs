use crate::api;
use crate::pages::editor::utils::{build_file_tree, FileEntry};
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub async fn sync_from_disk_async(pid: &str, ppath: &str, file_tree_data: RwSignal<Vec<FileEntry>>) {
    if let Ok(resp) = api::scan_project_api(ppath).await {
        if resp.error.is_empty() {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let prefix = format!("codedroid_file_{}_", pid);

                    // 1. Gather all existing local storage keys for this project
                    let mut local_keys = std::collections::HashSet::new();
                    let len = storage.length().unwrap_or(0);
                    for i in 0..len {
                        if let Ok(Some(k)) = storage.key(i) {
                            if k.starts_with(&prefix) {
                                local_keys.insert(k);
                            }
                        }
                    }

                    // 2. Add or update files/dirs from backend
                    let mut backend_keys = std::collections::HashSet::new();
                    for file in resp.files {
                        let key = if file.is_dir {
                            format!("{}{}/.codedroid_dir", prefix, file.rel_path)
                        } else {
                            format!("{}{}", prefix, file.rel_path)
                        };
                        backend_keys.insert(key.clone());

                        if !file.is_dir {
                            if !local_keys.contains(&key) {
                                // New file on disk: register placeholder (never write this to disk).
                                let _ = storage.set_item(&key, crate::store::UNLOADED_PLACEHOLDER);
                            } else if let Ok(Some(existing)) = storage.get_item(&key) {
                                // Migrate legacy empty placeholders that previously wiped files on reopen.
                                if existing.is_empty() {
                                    let _ = storage.set_item(&key, crate::store::UNLOADED_PLACEHOLDER);
                                }
                            }
                        }
                    }

                    // 3. Remove keys that exist locally but NOT on the backend (deleted files/dirs)
                    for key in local_keys {
                        if !backend_keys.contains(&key) {
                            let _ = storage.remove_item(&key);
                        }
                    }

                    // 4. Update the reactive file tree signal to trigger a UI re-render
                    file_tree_data.set(build_file_tree(pid));
                }
            }
        }
    }
}

pub fn sync_from_disk(pid: String, ppath: String, file_tree_data: RwSignal<Vec<FileEntry>>) {
    let p_id = pid.clone();
    let p_path = ppath.clone();
    spawn_local(async move {
        sync_from_disk_async(&p_id, &p_path, file_tree_data).await;
    });
}
