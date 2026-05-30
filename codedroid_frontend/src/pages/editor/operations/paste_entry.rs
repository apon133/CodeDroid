use crate::api;
use crate::pages::editor::utils::{build_file_tree, FileEntry};
use crate::store;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub fn make_paste_entry(
    pid: String,
    ppath: String,
    show_snack: Callback<String>,
    open_file: Callback<String>,
    file_tree_data: RwSignal<Vec<FileEntry>>,
    copied_item: RwSignal<Option<FileEntry>>,
) -> Callback<Option<String>> {
    Callback::new(move |target_dir: Option<String>| {
        if let Some(src_item) = copied_item.get_untracked() {
            let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
            let target_folder = target_dir.unwrap_or_default();

            // Determine new path
            let item_name = src_item.name.split('/').last().unwrap_or(&src_item.name);
            let mut dest_name = if target_folder.is_empty() {
                item_name.to_string()
            } else {
                format!("{}/{}", target_folder, item_name)
            };

            // Handle duplicates
            if dest_name == src_item.name {
                if src_item.is_dir {
                    dest_name = format!("{}_copy", dest_name);
                } else {
                    if let Some(idx) = dest_name.rfind('.') {
                        let (base, ext) = dest_name.split_at(idx);
                        dest_name = format!("{}_copy{}", base, ext);
                    } else {
                        dest_name = format!("{}_copy", dest_name);
                    }
                }
            }

            if src_item.is_dir {
                let len = storage.length().unwrap_or(0);
                let src_prefix = format!("codedroid_file_{}_{}/", pid, src_item.name);
                let mut copied_keys = Vec::new();

                for i in 0..len {
                    if let Ok(Some(k)) = storage.key(i) {
                        if k.starts_with(&src_prefix) {
                            if let Some(sub) = k.strip_prefix(&src_prefix) {
                                if let Ok(Some(val)) = storage.get_item(&k) {
                                    let new_k =
                                        format!("codedroid_file_{}_{}/{}", pid, dest_name, sub);
                                    copied_keys.push((new_k, val));
                                }
                            }
                        }
                    }
                }

                for (k, v) in copied_keys {
                    let _ = storage.set_item(&k, &v);
                }

                let src_marker = format!("codedroid_file_{}_{}/.codedroid_dir", pid, src_item.name);
                let dest_marker = format!("codedroid_file_{}_{}/.codedroid_dir", pid, dest_name);
                if let Ok(Some(_)) = storage.get_item(&src_marker) {
                    let _ = storage.set_item(&dest_marker, "");
                }

                // Sync to backend
                let src_full = format!("{}/{}", ppath, src_item.name);
                let dest_full = format!("{}/{}", ppath, dest_name);
                spawn_local(async move {
                    let _ = api::copy_file_api(&src_full, &dest_full, true).await;
                });
                show_snack.run(format!("Pasted folder as: {}", dest_name));
            } else {
                let src_key = store::file_key(&pid, &src_item.name);
                if let Ok(Some(content)) = storage.get_item(&src_key) {
                    let dest_key = store::file_key(&pid, &dest_name);
                    let _ = storage.set_item(&dest_key, &content);

                    // Sync to backend
                    let src_full = format!("{}/{}", ppath, src_item.name);
                    let dest_full = format!("{}/{}", ppath, dest_name);
                    let open_file = open_file.clone();
                    let dest_name_clone = dest_name.clone();
                    spawn_local(async move {
                        let _ = api::copy_file_api(&src_full, &dest_full, false).await;
                        open_file.run(dest_name_clone);
                    });
                    show_snack.run(format!("Pasted file as: {}", dest_name));
                }
            }

            // Refresh tree
            file_tree_data.set(build_file_tree(&pid));
        }
    })
}
