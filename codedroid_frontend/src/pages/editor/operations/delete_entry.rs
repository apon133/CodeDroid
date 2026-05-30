use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::pages::editor::utils::{build_file_tree, FileEntry};
use crate::store;

pub fn make_delete_entry(
    pid: String,
    ppath: String,
    show_snack: Callback<String>,
    close_tab: Callback<String>,
    file_tree_data: RwSignal<Vec<FileEntry>>,
) -> Callback<FileEntry> {
    Callback::new(move |entry: FileEntry| {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        
        if entry.is_dir {
            // Delete all keys in LocalStorage matching prefix
            let len = storage.length().unwrap_or(0);
            let dir_prefix = format!("codedroid_file_{}_{}/", pid, entry.name);
            let placeholder_key = format!("codedroid_file_{}_{}/.codedroid_dir", pid, entry.name);
            
            let mut keys_to_remove = Vec::new();
            for i in 0..len {
                if let Ok(Some(k)) = storage.key(i) {
                    if k.starts_with(&dir_prefix) || k == placeholder_key {
                        keys_to_remove.push(k.clone());
                        // Also close tab for any files in that directory
                        if let Some(rel) = k.strip_prefix(&format!("codedroid_file_{}_", pid)) {
                            close_tab.run(rel.to_string());
                        }
                    }
                }
            }
            for k in keys_to_remove {
                let _ = storage.remove_item(&k);
            }

            // Sync to backend
            let full_path = format!("{}/{}", ppath, entry.name);
            spawn_local(async move {
                let _ = api::delete_file_api(&full_path, true).await;
            });
            show_snack.run(format!("Deleted folder: {}", entry.name));
        } else {
            // Remove single file key
            let key = store::file_key(&pid, &entry.name);
            let _ = storage.remove_item(&key);
            close_tab.run(entry.name.clone());

            // Sync to backend
            let full_path = format!("{}/{}", ppath, entry.name);
            spawn_local(async move {
                let _ = api::delete_file_api(&full_path, false).await;
            });
            show_snack.run(format!("Deleted file: {}", entry.name));
        }

        // Refresh tree
        file_tree_data.set(build_file_tree(&pid));
    })
}
