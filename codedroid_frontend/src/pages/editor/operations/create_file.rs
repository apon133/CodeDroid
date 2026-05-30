use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::store;
use crate::pages::editor::utils::build_file_tree;

pub fn make_create_file(
    pid: String,
    ppath: String,
    show_snack: Callback<String>,
    open_file: Callback<String>,
    file_tree_data: RwSignal<Vec<crate::pages::editor::utils::FileEntry>>,
) -> Callback<String> {
    Callback::new(move |name: String| {
        let key = store::file_key(&pid, &name);
        store::save_file(&key, "// Start coding here...\n");
        
        // Sync to backend
        let full_path = format!("{}/{}", ppath, name);
        spawn_local(async move {
            let _ = api::save_file_api(&full_path, "// Start coding here...\n").await;
        });

        // Refresh tree
        file_tree_data.set(build_file_tree(&pid));
        show_snack.run(format!("Created file: {}", name));
        open_file.run(name);
    })
}
