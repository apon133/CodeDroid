use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::store;
use crate::pages::editor::utils::is_absolute_path;

pub fn make_save_current(
    pid: String,
    ppath: String,
    code: RwSignal<String>,
    dirty: RwSignal<bool>,
    active_tab: RwSignal<Option<String>>,
    trigger_diagnostics: Callback<String>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        if let Some(tab) = active_tab.get_untracked() {
            let key = store::file_key(&pid, &tab);
            let content = code.get_untracked();
            store::save_file(&key, &content);
            dirty.set(false);

            let base_path = ppath.clone();
            let tab_name = tab.clone();
            let trigger_diag_clone = trigger_diagnostics.clone();
            let content_clone = content.clone();
            spawn_local(async move {
                let full_path = if tab_name.starts_with('/') {
                    tab_name.clone()
                } else if tab_name.starts_with("Users/") || tab_name.starts_with("home/") || tab_name.starts_with("data/") {
                    format!("/{}", tab_name)
                } else if is_absolute_path(&tab_name) {
                    tab_name.clone()
                } else {
                    format!("{}/{}", base_path, tab_name)
                };
                let _ = api::save_file_api(&full_path, &content_clone).await;
                trigger_diag_clone.run(content_clone);
            });
        }
    })
}
