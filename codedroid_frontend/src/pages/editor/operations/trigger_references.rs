use crate::api;
use crate::pages::editor::utils::file_to_lsp_lang;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

pub fn make_trigger_references(
    code: RwSignal<String>,
    cursor_pos: RwSignal<u32>,
    project_path: String,
    active_tab: RwSignal<Option<String>>,
    references_list: RwSignal<Vec<api::Location>>,
    bottom_tab: RwSignal<usize>,
    show_snack: Callback<String>,
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

        let show_snack_cb = show_snack.clone();
        let ref_list_cb = references_list.clone();
        let b_tab_cb = bottom_tab.clone();

        spawn_local(async move {
            show_snack_cb.run("Finding references...".to_string());
            match api::get_references_api(&text, &lang, &path, &active_file, line, character).await
            {
                Ok(resp) => {
                    let locations = resp.locations;
                    if locations.is_empty() {
                        show_snack_cb.run("No references found".to_string());
                    } else {
                        ref_list_cb.set(locations.clone());
                        b_tab_cb.set(2);
                        show_snack_cb.run(format!("Found {} references", locations.len()));
                    }
                }
                Err(e) => {
                    show_snack_cb.run(format!("Error: {}", e));
                }
            }
        });
    })
}
