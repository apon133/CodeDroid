use crate::pages::editor::utils::pos_to_index;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub fn make_on_click_problem(
    open_file: Callback<String>,
    active_tab: RwSignal<Option<String>>,
    code: RwSignal<String>,
    check_error_at_cursor: Callback<(u32, u32)>,
    cursor_coords: RwSignal<(f64, f64)>,
) -> Callback<(Option<String>, u32, u32)> {
    Callback::new(
        move |(file_opt, line, character): (Option<String>, u32, u32)| {
            let open_file_clone = open_file.clone();
            let check_error_clone = check_error_at_cursor.clone();
            let cursor_coords_clone = cursor_coords.clone();
            let code_sig_clone = code.clone();
            let active_tab_clone = active_tab.clone();

            spawn_local(async move {
                if let Some(ref filename) = file_opt {
                    let current = active_tab_clone.get_untracked();
                    if current.as_ref() != Some(filename) {
                        open_file_clone.run(filename.clone());
                        gloo_timers::future::TimeoutFuture::new(50).await;
                    }
                }

                use wasm_bindgen::JsCast;
                if let Some(target) = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.query_selector(".code-editor").ok().flatten())
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
        },
    )
}
