use crate::api;
use crate::pages::editor::utils::{build_file_tree, FileEntry};
use crate::store;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub fn make_move_entry(
    pid: String,
    ppath: String,
    show_snack: Callback<String>,
    file_tree_data: RwSignal<Vec<FileEntry>>,
    active_tab: RwSignal<Option<String>>,
    open_tabs: RwSignal<Vec<String>>,
) -> Callback<(FileEntry, String)> {
    Callback::new(move |(entry, new_name): (FileEntry, String)| {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        let old_name = entry.name.clone();

        if old_name == new_name || new_name.trim().is_empty() {
            return;
        }

        if entry.is_dir {
            // Moving a directory: rename all keys in localStorage with matching prefix
            let len = storage.length().unwrap_or(0);
            let old_prefix = format!("codedroid_file_{}_{}/", pid, old_name);
            let new_prefix = format!("codedroid_file_{}_{}/", pid, new_name);
            let old_marker = format!("codedroid_file_{}_{}/.codedroid_dir", pid, old_name);
            let new_marker = format!("codedroid_file_{}_{}/.codedroid_dir", pid, new_name);

            let mut keys_to_move = Vec::new();
            for i in 0..len {
                if let Ok(Some(k)) = storage.key(i) {
                    if k.starts_with(&old_prefix) {
                        keys_to_move.push(k.clone());
                    }
                }
            }

            for k in keys_to_move {
                if let Ok(Some(val)) = storage.get_item(&k) {
                    let sub = k.strip_prefix(&old_prefix).unwrap();
                    let new_k = format!("{}{}", new_prefix, sub);
                    let _ = storage.set_item(&new_k, &val);
                    let _ = storage.remove_item(&k);
                }
            }

            if let Ok(Some(_)) = storage.get_item(&old_marker) {
                let _ = storage.set_item(&new_marker, "");
                let _ = storage.remove_item(&old_marker);
            }

            // Update open tabs
            open_tabs.update(|t| {
                for tab in t.iter_mut() {
                    if tab.starts_with(&format!("{}/", old_name)) {
                        if let Some(sub) = tab.strip_prefix(&format!("{}/", old_name)) {
                            *tab = format!("{}/{}", new_name, sub);
                        }
                    }
                }
            });

            // If active tab has changed name, update active_tab
            if let Some(active) = active_tab.get_untracked() {
                if active.starts_with(&format!("{}/", old_name)) {
                    if let Some(sub) = active.strip_prefix(&format!("{}/", old_name)) {
                        active_tab.set(Some(format!("{}/{}", new_name, sub)));
                    }
                }
            }

            // Sync to backend
            let src_full = format!("{}/{}", ppath, old_name);
            let dest_full = format!("{}/{}", ppath, new_name);
            spawn_local(async move {
                let _ = api::move_file_api(&src_full, &dest_full).await;
            });

            show_snack.run(format!("Moved folder to: {}", new_name));
        } else {
            // Moving a single file
            let old_key = store::file_key(&pid, &old_name);
            let new_key = store::file_key(&pid, &new_name);

            if let Ok(Some(content)) = storage.get_item(&old_key) {
                let _ = storage.set_item(&new_key, &content);
                let _ = storage.remove_item(&old_key);
            }

            // Update open tabs
            open_tabs.update(|t| {
                for tab in t.iter_mut() {
                    if *tab == old_name {
                        *tab = new_name.clone();
                    }
                }
            });

            // Update active tab
            if active_tab.get_untracked().as_deref() == Some(&old_name) {
                active_tab.set(Some(new_name.clone()));
            }

            // Sync to backend
            let src_full = format!("{}/{}", ppath, old_name);
            let dest_full = format!("{}/{}", ppath, new_name);
            spawn_local(async move {
                let _ = api::move_file_api(&src_full, &dest_full).await;
            });

            show_snack.run(format!("Moved file to: {}", new_name));
        }

        // Refresh tree
        file_tree_data.set(build_file_tree(&pid));
    })
}
