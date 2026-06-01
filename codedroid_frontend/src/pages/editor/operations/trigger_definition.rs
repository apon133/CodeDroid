use crate::api;
use crate::pages::editor::utils::{file_to_lsp_lang, is_absolute_path, pos_to_index};
use crate::store;
use gloo_storage::Storage;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub fn make_trigger_definition(
    pid: String,
    code: RwSignal<String>,
    cursor_pos: RwSignal<u32>,
    project_path: String,
    active_tab: RwSignal<Option<String>>,
    open_file: Callback<String>,
    references_list: RwSignal<Vec<crate::api::Location>>,
    bottom_tab: RwSignal<usize>,
    show_snack: Callback<String>,
    cursor_coords: RwSignal<(f64, f64)>,
    check_error_at_cursor: Callback<(u32, u32)>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        let active_file = match active_tab.get_untracked() {
            Some(f) => f,
            None => return,
        };
        let text = code.get_untracked();
        let pos = cursor_pos.get_untracked();

        let pos_usize = pos as usize;
        let safe_pos = pos_usize.min(text.len());
        let text_before = &text[..safe_pos];
        let lines: Vec<&str> = text_before.split('\n').collect();
        let line = lines.len().saturating_sub(1) as u32;
        let character = lines.last().map(|l| l.chars().count()).unwrap_or(0) as u32;

        let lang = file_to_lsp_lang(&active_file);
        let path = project_path.clone();

        let pid_clone = pid.clone();
        let open_file_cb = open_file.clone();
        let code_sig = code.clone();
        let check_error_cb = check_error_at_cursor.clone();
        let cursor_coords_cb = cursor_coords.clone();
        let show_snack_cb = show_snack.clone();
        let ref_list_cb = references_list.clone();
        let b_tab_cb = bottom_tab.clone();
        let active_tab_cb = active_tab.clone();
        let proj_path = path.clone();

        spawn_local(async move {
            show_snack_cb.run("Looking up definition...".to_string());
            match api::get_definition_api(&text, &lang, &path, &active_file, line, character).await
            {
                Ok(resp) => {
                    let locations = resp.locations;
                    if locations.is_empty() {
                        show_snack_cb.run("No definition found".to_string());
                    } else if locations.len() == 1 {
                        let loc = &locations[0];
                        let rel_path = crate::pages::editor::uri_to_relative(&loc.uri, &proj_path);
                        let target_line = loc.range.start.line;
                        let target_char = loc.range.start.character;

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

                        let current = active_tab_cb.get_untracked();
                        if current.as_ref() != Some(&rel_path) {
                            open_file_cb.run(rel_path.clone());
                            gloo_timers::future::TimeoutFuture::new(50).await;
                        }

                        use wasm_bindgen::JsCast;
                        if let Some(target) = web_sys::window()
                            .and_then(|w| w.document())
                            .and_then(|d| d.query_selector(".editor-pane.active .code-editor").ok().flatten().or_else(|| d.query_selector(".code-editor").ok().flatten()))
                        {
                            if let Ok(target) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                                let current_code = code_sig.get_untracked();
                                let index = pos_to_index(&current_code, target_line, target_char);
                                let _ = target.focus();
                                let _ = target.set_selection_range(index, index);

                                if let Some(mirror) = web_sys::window()
                                    .unwrap()
                                    .document()
                                    .unwrap()
                                    .get_element_by_id("cursor-mirror")
                                {
                                    let text_before = &current_code[..index as usize];
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
                                    cursor_coords_cb.set((
                                        span_el.offset_left() as f64,
                                        span_el.offset_top() as f64 + 20.0,
                                    ));
                                }
                                check_error_cb.run((target_line, target_char));
                            }
                        }
                        show_snack_cb.run(format!("Jumped to definition in {}", rel_path));
                    } else {
                        ref_list_cb.set(locations);
                        b_tab_cb.set(2);
                        show_snack_cb.run("Multiple definitions found".to_string());
                    }
                }
                Err(e) => {
                    show_snack_cb.run(format!("Error: {}", e));
                }
            }
        });
    })
}
