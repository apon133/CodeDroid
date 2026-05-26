use leptos::prelude::*;
use crate::models::Settings;
use crate::components::icon::LucideIcon;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn EditorFooter(
    code: RwSignal<String>,
    dirty: RwSignal<bool>,
    settings: RwSignal<Settings>,
    copy_code: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="editor-footer">
            {["TAB","{}","[]","()","\"\"","''","->","=>","::","/ /","/* */"].iter().map(|s| {
                let s_val = s.replace(" ", "");
                let s_val_2 = s_val.clone();
                let code_cb = code;
                let dirty_cb = dirty;
                let settings_cb = settings;
                
                view! {
                    <button class="btn btn-footer" on:click=move |_| {
                        let ins = if s_val_2 == "TAB" { " ".repeat(settings_cb.get_untracked().tab_size) } else { s_val_2.clone() };
                        use wasm_bindgen::JsCast;
                        if let Some(target) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.query_selector(".code-editor").ok().flatten()) {
                            if let Ok(target) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                                let start = target.selection_start().unwrap().unwrap_or(0);
                                let end = target.selection_end().unwrap().unwrap_or(0);
                                let val = js_sys::JsString::from(target.value());
                                code_cb.set(format!("{}{}{}", String::from(val.substring(0, start)), ins, String::from(val.substring(end, val.length()))));
                                dirty_cb.set(true);
                                let new_pos = start + ins.encode_utf16().count() as u32;
                                spawn_local(async move {
                                    let _ = gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                                    let _ = target.focus();
                                    let _ = target.set_selection_range(new_pos, new_pos);
                                });
                            }
                        }
                    }>{s_val}</button>
                }
            }).collect_view()}
            <div style="flex:1" />
            <button class="btn btn-footer" on:click=move |_| copy_code.run(())>
                <span style="display:inline-flex; align-items:center; gap:4px;"><LucideIcon name="copy" size="12" /> "Copy"</span>
            </button>
            <button class="btn btn-footer" on:click=move |_| { code.set(String::new()); dirty.set(true); }>
                <span style="display:inline-flex; align-items:center; gap:4px;"><LucideIcon name="trash" size="12" /> "Clear"</span>
            </button>
        </div>
    }
}
