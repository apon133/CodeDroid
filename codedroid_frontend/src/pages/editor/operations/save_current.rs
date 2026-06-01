use crate::api;
use crate::pages::editor::utils::is_absolute_path;
use crate::store;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use std::sync::{Arc, Mutex};

struct ThreadSafeTimeout(Option<gloo_timers::callback::Timeout>);
unsafe impl Send for ThreadSafeTimeout {}
unsafe impl Sync for ThreadSafeTimeout {}

pub fn make_save_current(
    pid: String,
    ppath: String,
    code: RwSignal<String>,
    dirty: RwSignal<bool>,
    active_tab: RwSignal<Option<String>>,
    trigger_diagnostics: Callback<String>,
) -> Callback<bool> {
    let pending_timeout = Arc::new(Mutex::new(ThreadSafeTimeout(None)));

    Callback::new(move |is_manual: bool| {
        if let Some(tab) = active_tab.get_untracked() {
            let key = store::file_key(&pid, &tab);
            let content = code.get_untracked();
            
            // Immediately save to browser local storage cache
            store::save_file(&key, &content);
            dirty.set(false);

            // Cancel any pending debounced backend save
            if let Some(timeout) = pending_timeout.lock().unwrap().0.take() {
                timeout.cancel();
            }

            let base_path = ppath.clone();
            let tab_name = tab.clone();
            let trigger_diag_clone = trigger_diagnostics.clone();
            let content_clone = content.clone();

            let save_to_disk = move || {
                if store::should_skip_disk_sync(&content_clone) {
                    return;
                }
                let base_path = base_path.clone();
                let tab_name = tab_name.clone();
                let trigger_diag_clone = trigger_diag_clone.clone();
                let content_clone = content_clone.clone();
                spawn_local(async move {
                    let full_path = if tab_name.starts_with('/') {
                        tab_name.clone()
                    } else if tab_name.starts_with("Users/")
                        || tab_name.starts_with("home/")
                        || tab_name.starts_with("data/")
                    {
                        format!("/{}", tab_name)
                    } else if is_absolute_path(&tab_name) {
                        tab_name.clone()
                    } else {
                        format!("{}/{}", base_path, tab_name)
                    };
                    let _ = api::save_file_api(&full_path, &content_clone).await;
                    trigger_diag_clone.run(content_clone);
                });
            };

            if is_manual {
                save_to_disk();
            } else {
                let pending_timeout_clone = pending_timeout.clone();
                let timeout = gloo_timers::callback::Timeout::new(500, move || {
                    pending_timeout_clone.lock().unwrap().0.take();
                    save_to_disk();
                });
                pending_timeout.lock().unwrap().0 = Some(timeout);
            }
        }
    })
}
