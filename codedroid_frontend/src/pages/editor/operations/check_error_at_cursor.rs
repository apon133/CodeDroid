use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;

pub fn make_check_error_at_cursor(
    code: RwSignal<String>,
    diagnostics_list: RwSignal<Vec<api::Diagnostic>>,
    project_lang: String,
    active_error: RwSignal<Option<(api::Diagnostic, Vec<api::CodeSuggestion>, bool)>>,
    active_tab: RwSignal<Option<String>>,
) -> Callback<(u32, u32)> {
    Callback::new(move |(line, col): (u32, u32)| {
        let diags = diagnostics_list.get_untracked();
        let current_tab = active_tab.get_untracked();

        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
            "[check_error_at_cursor] Clicked/Cursor at line: {}, col: {}. Active Tab: {:?}. Total Diags: {}",
            line, col, current_tab, diags.len()
        )));

        let diag_opt = diags.iter().find(|d| {
            let file_matches = d.file.is_none() || d.file == current_tab;
            if !file_matches { return false; }
            if line >= d.range.start.line && line <= d.range.end.line {
                if line == d.range.start.line && line == d.range.end.line {
                    col >= d.range.start.character && col <= d.range.end.character
                } else if line == d.range.start.line {
                    col >= d.range.start.character
                } else if line == d.range.end.line {
                    col <= d.range.end.character
                } else {
                    true
                }
            } else {
                false
            }
        }).cloned().or_else(|| {
            diags.iter().find(|d| {
                let file_matches = d.file.is_none() || d.file == current_tab;
                file_matches && line >= d.range.start.line && line <= d.range.end.line
            }).cloned()
        });

        if let Some(diag) = diag_opt {
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                "[check_error_at_cursor] Matching diagnostic found: {}", diag.message
            )));

            let current = active_error.get_untracked();
            if let Some((curr_diag, _, _)) = &current {
                if curr_diag.message == diag.message && curr_diag.range.start.line == diag.range.start.line {
                    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
                        "[check_error_at_cursor] Diagnostic already active. Skipping fetch."
                    ));
                    return;
                }
            }

            active_error.set(Some((diag.clone(), Vec::new(), true)));

            let code_val = code.get_untracked();
            let lang_val = project_lang.clone();

            spawn_local(async move {
                web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
                    "[check_error_at_cursor] Fetching error suggestions from API..."
                ));
                if let Ok(resp) = api::get_error_suggestions_api(&code_val, &lang_val, &diag).await {
                    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                        "[check_error_at_cursor] API call success. Got {} suggestions.", resp.suggestions.len()
                    )));
                    active_error.set(Some((diag, resp.suggestions, false)));
                } else {
                    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
                        "[check_error_at_cursor] API call failed. Setting active_error with empty suggestions."
                    ));
                    active_error.set(Some((diag, Vec::new(), false)));
                }
            });
        } else {
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
                "[check_error_at_cursor] No matching diagnostic found. Setting active_error to None."
            ));
            active_error.set(None);
        }
    })
}
