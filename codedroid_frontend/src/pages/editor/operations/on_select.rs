use crate::api;
use crate::pages::editor::utils::resolve_completion;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub fn make_on_select(
    code: RwSignal<String>,
    dirty: RwSignal<bool>,
    suggestions: RwSignal<Vec<api::CompletionItem>>,
    cursor_pos: RwSignal<u32>,
) -> Callback<api::CompletionItem> {
    Callback::new(move |item: api::CompletionItem| {
        let cpos = cursor_pos.get_untracked();
        use wasm_bindgen::JsCast;
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            let target_opt = doc.query_selector(".editor-pane.active .code-editor")
                .ok()
                .flatten()
                .or_else(|| doc.query_selector(".code-editor").ok().flatten());
            if let Some(target) = target_opt {
                if let Ok(target) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                    let start = target.selection_start().unwrap().unwrap_or(cpos);
                    let end = target.selection_end().unwrap().unwrap_or(cpos);
                    let val = js_sys::JsString::from(target.value());
                    let rust_val = String::from(val.clone());
                    let mut word_start = start as usize;
                    let chars_vec: Vec<char> = rust_val.chars().take(start as usize).collect();
                    for (i, c) in chars_vec.into_iter().enumerate().rev() {
                        if !c.is_alphanumeric() && c != '_' {
                            word_start = i + 1;
                            break;
                        }
                        if i == 0 {
                            word_start = 0;
                        }
                    }
                    let before = val.substring(0, word_start as u32);
                    let after = val.substring(end, val.length());

                    let (ins, cursor_offset) = resolve_completion(&item);

                    let new_val = format!("{}{}{}", String::from(before), ins, String::from(after));
                    let new_pos = if let Some(offset) = cursor_offset {
                        word_start as u32 + offset as u32
                    } else {
                        word_start as u32 + ins.encode_utf16().count() as u32
                    };

                    code.set(new_val);
                    dirty.set(true);
                    suggestions.set(Vec::new());

                    spawn_local(async move {
                        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                            let target_opt = doc.query_selector(".editor-pane.active .code-editor")
                                .ok()
                                .flatten()
                                .or_else(|| doc.query_selector(".code-editor").ok().flatten());
                            if let Some(target) = target_opt {
                                if let Ok(target) =
                                    target.dyn_into::<web_sys::HtmlTextAreaElement>()
                                {
                                    let _ = target.focus();
                                    target.set_selection_start(Some(new_pos)).unwrap();
                                    target.set_selection_end(Some(new_pos)).unwrap();
                                }
                            }
                        }
                    });
                }
            }
        }
    })
}
