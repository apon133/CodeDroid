use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::pages::editor::utils::{file_to_lsp_lang, is_project_source_file};

pub fn make_trigger_diagnostics(
    ppath: String,
    lang: String,
    diagnostics_list: RwSignal<Vec<api::Diagnostic>>,
    last_diag_req: RwSignal<u64>,
    active_tab: RwSignal<Option<String>>,
) -> Callback<String> {
    Callback::new(move |code_val: String| {
        let filename = match active_tab.get_untracked() {
            Some(f) => f,
            None => {
                diagnostics_list.set(Vec::new());
                return;
            }
        };
        if !is_project_source_file(&filename, &lang) {
            diagnostics_list.set(Vec::new());
            return;
        }
        let file_lang = file_to_lsp_lang(&filename);
        let ppath = ppath.clone();
        let req_id = last_diag_req.get_untracked() + 1;
        last_diag_req.set(req_id);
        let rel_file = filename.clone();
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(800).await;
            if last_diag_req.get_untracked() == req_id {
                if let Ok(resp) = api::get_diagnostics_api(&code_val, &file_lang, &ppath, &rel_file).await {
                    if last_diag_req.get_untracked() == req_id {
                        diagnostics_list.set(resp.diagnostics);
                    }
                }
            }
        });
    })
}
