use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_storage::Storage;
use crate::api;
use crate::store;
use crate::pages::editor::utils::is_absolute_path;

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
        let key = store::file_key(&pid, &name);
        let content = store::load_file(&key);
        open_tabs.update(|t| { if !t.contains(&name) { t.push(name.clone()); }});
        active_tab.set(Some(name.clone()));
        code.set(content.clone());
        dirty.set(false);
        trigger_diagnostics.run(content.clone());

        let key_exists = gloo_storage::LocalStorage::get::<String>(&key).is_ok();
        let is_absolute = is_absolute_path(&name);
        if !key_exists || is_absolute || content.is_empty() {
            let name_clone = name.clone();
            let key_clone = key.clone();
            let ppath_clone = ppath.clone();
            let code_clone = code.clone();
            let active_tab_clone = active_tab.clone();
            let trigger_diag_clone = trigger_diagnostics.clone();

            spawn_local(async move {
                let file_path = if name_clone.starts_with('/') {
                    name_clone.clone()
                } else if name_clone.starts_with("Users/") || name_clone.starts_with("home/") || name_clone.starts_with("data/") {
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
