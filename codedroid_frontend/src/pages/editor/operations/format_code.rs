use crate::api;
use crate::pages::editor::utils::file_to_lsp_lang;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub fn make_format_code(
    ppath: String,
    plang: String,
    code: RwSignal<String>,
    dirty: RwSignal<bool>,
    active_tab: RwSignal<Option<String>>,
    output: RwSignal<String>,
    is_error: RwSignal<bool>,
    bottom_tab: RwSignal<usize>,
    trigger_diagnostics: Callback<String>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        let current_code = code.get_untracked();
        if current_code.trim().is_empty() {
            return;
        }

        let lang = if let Some(ref filename) = active_tab.get_untracked() {
            file_to_lsp_lang(filename)
        } else {
            plang.clone()
        };

        let path = ppath.clone();
        let trigger_diag = trigger_diagnostics.clone();

        spawn_local(async move {
            let res = api::format_code_api(&current_code, &lang, &path).await;
            match res {
                Ok(r) => {
                    if let Some(err) = r.error {
                        output.set(format!("⚠️ Formatting Warning/Error:\n{}", err));
                        is_error.set(true);
                        bottom_tab.set(0);
                    } else {
                        code.set(r.formatted_code.clone());
                        trigger_diag.run(r.formatted_code);
                        dirty.set(true);
                    }
                }
                Err(e) => {
                    output.set(format!("❌ Formatting connection error:\n{}", e));
                    is_error.set(true);
                    bottom_tab.set(0);
                }
            }
        });
    })
}
