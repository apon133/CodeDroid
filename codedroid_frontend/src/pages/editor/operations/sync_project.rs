use crate::api;
use crate::pages::editor::utils::is_absolute_path;
use crate::store;
use wasm_bindgen_futures::spawn_local;

pub async fn sync_project_async(pid: &str, ppath: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let len = storage.length().unwrap_or(0);
            let prefix = format!("codedroid_file_{}_", pid);
            for i in 0..len {
                if let Ok(Some(k)) = storage.key(i) {
                    if let Some(rel) = k.strip_prefix(&prefix) {
                        if is_absolute_path(rel) {
                            continue;
                        }
                        if rel.ends_with("/.codedroid_dir") {
                            let dir_name = rel.trim_end_matches("/.codedroid_dir");
                            if !dir_name.is_empty() {
                                let full_dir_path = format!("{}/{}", ppath, dir_name);
                                let _ = api::create_dir_api(&full_dir_path).await;
                            }
                        } else if !rel.is_empty() {
                            let content = store::load_file(&k);
                            let full_file_path = format!("{}/{}", ppath, rel);
                            let _ = api::save_file_api(&full_file_path, &content).await;
                        }
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn sync_project(pid: String, ppath: String) {
    spawn_local(async move {
        sync_project_async(&pid, &ppath).await;
    });
}
