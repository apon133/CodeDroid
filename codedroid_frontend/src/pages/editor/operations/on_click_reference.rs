use crate::api;
use crate::pages::editor::utils::{is_absolute_path, pos_to_index};
use crate::store;
use gloo_storage::Storage;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub fn make_on_click_reference(
    pid: String,
    open_file: Callback<String>,
    active_tab: RwSignal<Option<String>>,
    code: RwSignal<String>,
    check_error_at_cursor: Callback<(u32, u32)>,
    cursor_coords: RwSignal<(f64, f64)>,
    project_path: String,
) -> Callback<api::Location> {
    Callback::new(move |loc: api::Location| {
        let pid_clone = pid.clone();
        let open_file_clone = open_file.clone();
        let check_error_clone = check_error_at_cursor.clone();
        let cursor_coords_clone = cursor_coords.clone();
        let code_sig_clone = code.clone();
        let active_tab_clone = active_tab.clone();
        let proj_path = project_path.clone();

        spawn_local(async move {
            let rel_path = crate::pages::editor::uri_to_relative(&loc.uri, &proj_path);
            let line = loc.range.start.line;
            let character = loc.range.start.character;

            let key = store::file_key(&pid_clone, &rel_path);
            let key_exists = gloo_storage::LocalStorage::get::<String>(&key).is_ok();
            let is_absolute = is_absolute_path(&rel_path);
            let content_is_empty = store::needs_load_from_disk(&store::load_file(&key));

            if !key_exists || is_absolute || content_is_empty {
                let file_path = if rel_path.starts_with('/') {
                    rel_path.clone()
                } else if rel_path.starts_with("Users/")
                    || rel_path.starts_with("home/")
                    || rel_path.starts_with("data/")
                {
                    format!("/{}", rel_path)
                } else if is_absolute_path(&rel_path) {
                    rel_path.clone()
                } else {
                    format!("{}/{}", proj_path, rel_path)
                };

                if let Ok(resp) = api::read_file_api(&file_path).await {
                    if resp.error.is_empty() {
                        store::save_file(&key, &resp.content);
                    }
                }
            }

            let current = active_tab_clone.get_untracked();
            if current.as_ref() != Some(&rel_path) {
                open_file_clone.run(rel_path.clone());
                gloo_timers::future::TimeoutFuture::new(50).await;
            }

            use wasm_bindgen::JsCast;
            if let Some(target) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.query_selector(".editor-pane.active .code-editor").ok().flatten().or_else(|| d.query_selector(".code-editor").ok().flatten()))
            {
                if let Ok(target) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                    let text = code_sig_clone.get_untracked();
                    let index = pos_to_index(&text, line, character);
                    let _ = target.focus();
                    let _ = target.set_selection_range(index, index);

                    if let Some(mirror) = web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .get_element_by_id("cursor-mirror")
                    {
                        let text_before = &text[..index as usize];
                        mirror.set_text_content(Some(text_before));
                        let span = web_sys::window()
                            .unwrap()
                            .document()
                            .unwrap()
                            .create_element("span")
                            .unwrap();
                        span.set_text_content(Some("|"));
                        let _ = mirror.append_child(&span);
                        let span_el = span.dyn_into::<web_sys::HtmlElement>().unwrap();
                        cursor_coords_clone.set((
                            span_el.offset_left() as f64,
                            span_el.offset_top() as f64 + 20.0,
                        ));
                    }
                    check_error_clone.run((line, character));
                }
            }
        });
    })
}
