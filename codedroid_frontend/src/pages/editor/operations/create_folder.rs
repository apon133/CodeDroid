use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::pages::editor::utils::build_file_tree;

pub fn make_create_folder(
    pid: String,
    ppath: String,
    show_snack: Callback<String>,
    file_tree_data: RwSignal<Vec<crate::pages::editor::utils::FileEntry>>,
) -> Callback<String> {
    Callback::new(move |name: String| {
        let key = format!("codedroid_file_{}_{}/.codedroid_dir", pid, name);
        crate::store::save_file(&key, "");

        // Sync to backend
        let full_path = format!("{}/{}", ppath, name);
        spawn_local(async move {
            let _ = api::create_dir_api(&full_path).await;
        });

        // Refresh tree
        file_tree_data.set(build_file_tree(&pid));
        show_snack.run(format!("Created folder: {}", name));
    })
}
