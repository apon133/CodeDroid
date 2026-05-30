use crate::api;
use crate::pages::editor::utils::build_file_tree;
use crate::store;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub fn make_add_dep(
    pid: String,
    ppath: String,
    plang: String,
    dep_input: RwSignal<String>,
    dep_output: RwSignal<String>,
    open_file: Callback<String>,
    file_tree_data: RwSignal<Vec<crate::pages::editor::utils::FileEntry>>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        let pkg = dep_input.get_untracked();
        if pkg.trim().is_empty() {
            return;
        }
        let path = ppath.clone();
        let lang = plang.clone();
        let pid_clone = pid.clone();
        let open_file_clone = open_file.clone();
        let file_tree_data_clone = file_tree_data.clone();
        dep_output.set(format!("Installing {}...", pkg));
        spawn_local(async move {
            match api::add_package(&pkg, &lang, &path).await {
                Ok(r) => {
                    dep_output.set(if r.error.is_empty() {
                        r.output
                    } else {
                        r.error
                    });
                    if let (Some(filename), Some(content)) =
                        (r.dependency_file_name, r.dependency_file_content)
                    {
                        let key = store::file_key(&pid_clone, &filename);
                        store::save_file(&key, &content);
                        file_tree_data_clone.set(build_file_tree(&pid_clone));
                        open_file_clone.run(filename);
                    }
                }
                Err(e) => dep_output.set(format!("Error: {e}")),
            }
        });
    })
}
