use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use wasm_bindgen_futures::spawn_local;

pub mod utils;
pub mod components;
pub mod preview;
pub mod footer;
pub mod search_bar;
pub mod code_area;
pub mod error_popover;
pub mod hover;
pub mod suggestions;

use utils::*;
use components::*;
use code_area::EditorCodeArea;
use preview::*;
use footer::*;
use search_bar::*;
use crate::models::{Project, Settings, lang_icon};
use crate::store;
use crate::api;
use gloo_storage::Storage;
use crate::components::app_bar::AppBar;
use crate::components::snackbar::Snackbar;
use crate::components::icon::LucideIcon;

#[component]
pub fn EditorPage() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();

    // Resolve project
    let projects = store::load_projects();
    let project_id = move || params.get().get("id").unwrap_or_default().clone();
    let project: Option<Project> = {
        let id = project_id();
        projects.into_iter().find(|p| p.id == id)
    };

    if project.is_none() {
        let nav = navigate.clone();
        nav("/", Default::default());
        return view! { <div>"Redirecting..."</div> }.into_any();
    }
    let project = project.unwrap();
    let project_lang_str = StoredValue::new(project.language.clone());
    let project_path_str = StoredValue::new(project.path.clone());

    // State
    let settings: RwSignal<Settings> = RwSignal::new(store::load_settings());
    let open_tabs: RwSignal<Vec<String>> = RwSignal::new(Vec::new());
    let active_tab: RwSignal<Option<String>> = RwSignal::new(None);
    let code: RwSignal<String> = RwSignal::new(String::new());
    let dirty: RwSignal<bool> = RwSignal::new(false);
    let output: RwSignal<String> = RwSignal::new("// Output will appear here...".to_string());
    let is_error: RwSignal<bool> = RwSignal::new(false);
    let is_running: RwSignal<bool> = RwSignal::new(false);
    let current_pid: RwSignal<Option<u32>> = RwSignal::new(None);
    let preview_url: RwSignal<Option<String>> = RwSignal::new(None);
    let bottom_tab: RwSignal<usize> = RwSignal::new(0); // 0=terminal 1=preview
    let show_mobile_full_preview: RwSignal<bool> = RwSignal::new(false);
    let refresh_key: RwSignal<u32> = RwSignal::new(0);
    let show_search: RwSignal<bool> = RwSignal::new(false);
    let find_text: RwSignal<String> = RwSignal::new(String::new());
    let find_index: RwSignal<usize> = RwSignal::new(0);
    let project_search_text: RwSignal<String> = RwSignal::new(String::new());
    let project_replace_text: RwSignal<String> = RwSignal::new(String::new());
    let snack_msg: RwSignal<Option<String>> = RwSignal::new(None);
    let file_tree_data: RwSignal<Vec<FileEntry>> = RwSignal::new(build_file_tree(&project.id));
    let show_deps: RwSignal<bool> = RwSignal::new(false);
    let show_more_menu: RwSignal<bool> = RwSignal::new(false);
    let dep_input: RwSignal<String> = RwSignal::new(String::new());
    let dep_output: RwSignal<String> = RwSignal::new(String::new());
    let suggestions = RwSignal::new(Vec::<api::CompletionItem>::new());
    let selected_idx = RwSignal::new(0);
    let cursor_pos = RwSignal::new(0);
    let cursor_coords = RwSignal::new((0.0, 0.0));
    let last_request_id = RwSignal::new(0u64);
    let diagnostics_list = RwSignal::new(Vec::<api::Diagnostic>::new());
    let references_list = RwSignal::new(Vec::<crate::api::Location>::new());
    let last_diag_req = RwSignal::new(0u64);
    let active_error = RwSignal::new(Option::<(api::Diagnostic, Vec<api::CodeSuggestion>, bool)>::None);

    // Callbacks
    let show_snack = Callback::new({
        let snack = snack_msg;
        move |msg: String| {
            snack.set(Some(msg));
            let s2 = snack;
            gloo_timers::callback::Timeout::new(3000, move || s2.set(None)).forget();
        }
    });

    let trigger_diagnostics = Callback::new({
        let ppath = project.path.clone();
        let lang = project.language.clone();
        let diagnostics_list = diagnostics_list.clone();
        let last_diag_req = last_diag_req.clone();
        let active_tab = active_tab.clone();
        move |code_val: String| {
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
        }
    });

    let check_error_at_cursor = Callback::new({
        let code_sig = code;
        let diagnostics_list = diagnostics_list.clone();
        let project_lang_str = project_lang_str.clone();
        let active_error = active_error.clone();
        let active_tab = active_tab.clone();
        move |(line, col): (u32, u32)| {
            let diags = diagnostics_list.get_untracked();
            let current_tab = active_tab.get_untracked();
            
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                "[check_error_at_cursor] Clicked/Cursor at line: {}, col: {}. Active Tab: {:?}. Total Diags: {}",
                line, col, current_tab, diags.len()
            )));

            let diag_opt = diags.iter().find(|d| {
                let file_matches = d.file.is_none() || d.file == current_tab;
                if !file_matches { return false; }
                
                // Precise match: cursor is within the diagnostic range (lines and columns)
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
                // Line-level fallback: cursor is on any line intersected by the diagnostic
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
                
                let code_val = code_sig.get_untracked();
                let lang_val = project_lang_str.get_value();
                
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
        }
    });

    let pid = project.id.clone();
    let ppath_val = project.path.clone();
    let open_file = Callback::new({
        let pid = pid.clone();
        let ppath_val = ppath_val.clone();
        let code = code.clone();
        let active_tab = active_tab.clone();
        let open_tabs = open_tabs.clone();
        let dirty = dirty.clone();
        let trigger_diag = trigger_diagnostics.clone();
        move |name: String| {
            let key = store::file_key(&pid, &name);
            let content = store::load_file(&key);
            open_tabs.update(|t| { if !t.contains(&name) { t.push(name.clone()); }});
            active_tab.set(Some(name.clone()));
            code.set(content.clone());
            dirty.set(false);
            trigger_diag.run(content.clone());

            let key_exists = gloo_storage::LocalStorage::get::<String>(&key).is_ok();
            let is_absolute = name.starts_with('/') || name.starts_with("Users/") || name.starts_with("home/") || name.starts_with("data/");
            if !key_exists || is_absolute || content.is_empty() {
                let name_clone = name.clone();
                let key_clone = key.clone();
                let ppath_clone = ppath_val.clone();
                let code_clone = code.clone();
                let active_tab_clone = active_tab.clone();
                let trigger_diag_clone = trigger_diag.clone();
                
                spawn_local(async move {
                    let file_path = if name_clone.starts_with('/') {
                        name_clone.clone()
                    } else if name_clone.starts_with("Users/") || name_clone.starts_with("home/") || name_clone.starts_with("data/") {
                        format!("/{}", name_clone)
                    } else {
                        format!("{}/{}", ppath_clone, name_clone)
                    };
                    
                    if let Ok(resp) = api::read_file_api(&file_path).await {
                        if resp.error.is_empty() {
                            store::save_file(&key_clone, &resp.content);
                            if active_tab_clone.get_untracked().as_ref() == Some(&name_clone) {
                                code_clone.set(resp.content.clone());
                                trigger_diag_clone.run(resp.content);
                            }
                        }
                    }
                });
            }
        }
    });

    let on_click_problem = Callback::new({
        let open_file = open_file.clone();
        let active_tab = active_tab.clone();
        let code_signal = code;
        let check_error = check_error_at_cursor.clone();
        let cursor_coords = cursor_coords.clone();
        move |(file_opt, line, character): (Option<String>, u32, u32)| {
            let open_file_clone = open_file.clone();
            let check_error_clone = check_error.clone();
            let cursor_coords_clone = cursor_coords.clone();
            let code_sig_clone = code_signal.clone();
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
                        
                        if let Some(mirror) = web_sys::window().unwrap().document().unwrap().get_element_by_id("cursor-mirror") {
                            let text_before = &text[..index as usize];
                            mirror.set_text_content(Some(text_before));
                            let span = web_sys::window().unwrap().document().unwrap().create_element("span").unwrap();
                            span.set_text_content(Some("|"));
                            let _ = mirror.append_child(&span);
                            let span_el = span.dyn_into::<web_sys::HtmlElement>().unwrap();
                            cursor_coords_clone.set((span_el.offset_left() as f64, span_el.offset_top() as f64 + 20.0));
                        }
                        
                        check_error_clone.run((line, character));
                    }
                }
            });
        }
    });

    let ppath = project.path.clone();
    let save_current = Callback::new({
        let pid = pid.clone();
        let ppath = ppath.clone();
        let trigger_diag = trigger_diagnostics.clone();
        move |_: ()| {
            if let Some(tab) = active_tab.get_untracked() {
                let key = store::file_key(&pid, &tab);
                let content = code.get_untracked();
                store::save_file(&key, &content);
                dirty.set(false);

                let base_path = ppath.clone();
                let tab_name = tab.clone();
                let trigger_diag_clone = trigger_diag.clone();
                let content_clone = content.clone();
                spawn_local(async move {
                    let full_path = format!("{}/{}", base_path, tab_name);
                    let _ = api::save_file_api(&full_path, &content_clone).await;
                    trigger_diag_clone.run(content_clone);
                });
            }
        }
    });

    let close_tab = Callback::new({
        let pid = pid.clone();
        move |name: String| {
            open_tabs.update(|t| t.retain(|n| *n != name));
            let tabs = open_tabs.get_untracked();
            if active_tab.get_untracked().as_deref() == Some(&name) {
                if let Some(first) = tabs.first() {
                    let key = store::file_key(&pid, first);
                    code.set(store::load_file(&key));
                    active_tab.set(Some(first.clone()));
                } else {
                    active_tab.set(None);
                    code.set(String::new());
                }
            }
        }
    });

    let trigger_definition = Callback::new({
        let pid = pid.clone();
        let code = code.clone();
        let cursor_pos = cursor_pos.clone();
        let project_path_str = project_path_str.clone();
        let active_tab = active_tab.clone();
        let open_file = open_file.clone();
        let references_list = references_list.clone();
        let bottom_tab = bottom_tab.clone();
        let show_snack = show_snack.clone();
        let cursor_coords = cursor_coords.clone();
        let check_error = check_error_at_cursor.clone();
        move |_: ()| {
            let active_file = match active_tab.get_untracked() {
                Some(f) => f,
                None => return,
            };
            let text = code.get_untracked();
            let pos = cursor_pos.get_untracked();
            
            let pos_usize = pos as usize;
            let text_len = text.len();
            let safe_pos = if pos_usize > text_len { text_len } else { pos_usize };
            let text_before = &text[..safe_pos];
            let lines: Vec<&str> = text_before.split('\n').collect();
            let line = lines.len().saturating_sub(1) as u32;
            let character = lines.last().map(|l| l.chars().count()).unwrap_or(0) as u32;
            
            let lang = file_to_lsp_lang(&active_file);
            let path = project_path_str.get_value();
            
            let pid_clone = pid.clone();
            let open_file_cb = open_file.clone();
            let code_sig = code.clone();
            let check_error_cb = check_error.clone();
            let cursor_coords_cb = cursor_coords.clone();
            let show_snack_cb = show_snack.clone();
            let ref_list_cb = references_list.clone();
            let b_tab_cb = bottom_tab.clone();
            let active_tab_cb = active_tab.clone();
            let proj_path = path.clone();
            
            spawn_local(async move {
                show_snack_cb.run("Looking up definition...".to_string());
                match api::get_definition_api(&text, &lang, &path, &active_file, line, character).await {
                    Ok(resp) => {
                        let locations = resp.locations;
                        if locations.is_empty() {
                            show_snack_cb.run("No definition found".to_string());
                        } else if locations.len() == 1 {
                            let loc = &locations[0];
                            let rel_path = uri_to_relative(&loc.uri, &proj_path);
                            let target_line = loc.range.start.line;
                            let target_char = loc.range.start.character;
                            
                            // Pre-fetch file from backend if not cached in LocalStorage
                            let key = store::file_key(&pid_clone, &rel_path);
                            let key_exists = gloo_storage::LocalStorage::get::<String>(&key).is_ok();
                            let is_absolute = rel_path.starts_with('/') || rel_path.starts_with("Users/") || rel_path.starts_with("home/") || rel_path.starts_with("data/");
                            let content_is_empty = store::load_file(&key).is_empty();
                            
                            if !key_exists || is_absolute || content_is_empty {
                                let file_path = if rel_path.starts_with('/') {
                                    rel_path.clone()
                                } else if rel_path.starts_with("Users/") || rel_path.starts_with("home/") || rel_path.starts_with("data/") {
                                    format!("/{}", rel_path)
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
                                .and_then(|d| d.query_selector(".code-editor").ok().flatten())
                            {
                                if let Ok(target) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                                    let current_code = code_sig.get_untracked();
                                    let index = pos_to_index(&current_code, target_line, target_char);
                                    let _ = target.focus();
                                    let _ = target.set_selection_range(index, index);
                                    
                                    if let Some(mirror) = web_sys::window().unwrap().document().unwrap().get_element_by_id("cursor-mirror") {
                                        let text_before = &current_code[..index as usize];
                                        mirror.set_text_content(Some(text_before));
                                        let span = web_sys::window().unwrap().document().unwrap().create_element("span").unwrap();
                                        span.set_text_content(Some("|"));
                                        let _ = mirror.append_child(&span);
                                        let span_el = span.dyn_into::<web_sys::HtmlElement>().unwrap();
                                        cursor_coords_cb.set((span_el.offset_left() as f64, span_el.offset_top() as f64 + 20.0));
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
        }
    });

    let trigger_references = Callback::new({
        let code = code.clone();
        let cursor_pos = cursor_pos.clone();
        let project_path_str = project_path_str.clone();
        let active_tab = active_tab.clone();
        let references_list = references_list.clone();
        let bottom_tab = bottom_tab.clone();
        let show_snack = show_snack.clone();
        move |_: ()| {
            let active_file = match active_tab.get_untracked() {
                Some(f) => f,
                None => return,
            };
            let text = code.get_untracked();
            let pos = cursor_pos.get_untracked();
            
            let pos_usize = pos as usize;
            let text_len = text.len();
            let safe_pos = if pos_usize > text_len { text_len } else { pos_usize };
            let text_before = &text[..safe_pos];
            let lines: Vec<&str> = text_before.split('\n').collect();
            let line = lines.len().saturating_sub(1) as u32;
            let character = lines.last().map(|l| l.chars().count()).unwrap_or(0) as u32;
            
            let lang = file_to_lsp_lang(&active_file);
            let path = project_path_str.get_value();
            
            let show_snack_cb = show_snack.clone();
            let ref_list_cb = references_list.clone();
            let b_tab_cb = bottom_tab.clone();
            
            spawn_local(async move {
                show_snack_cb.run("Finding references...".to_string());
                match api::get_references_api(&text, &lang, &path, &active_file, line, character).await {
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
        }
    });

    let on_click_reference = Callback::new({
        let pid = pid.clone();
        let open_file = open_file.clone();
        let active_tab = active_tab.clone();
        let code_signal = code;
        let check_error = check_error_at_cursor.clone();
        let cursor_coords = cursor_coords.clone();
        let project_path_str = project_path_str.clone();
        move |loc: crate::api::Location| {
            let pid_clone = pid.clone();
            let open_file_clone = open_file.clone();
            let check_error_clone = check_error.clone();
            let cursor_coords_clone = cursor_coords.clone();
            let code_sig_clone = code_signal.clone();
            let active_tab_clone = active_tab.clone();
            let proj_path = project_path_str.get_value();
            
            spawn_local(async move {
                let rel_path = uri_to_relative(&loc.uri, &proj_path);
                let line = loc.range.start.line;
                let character = loc.range.start.character;
                
                // Pre-fetch file from backend if not cached in LocalStorage
                let key = store::file_key(&pid_clone, &rel_path);
                let key_exists = gloo_storage::LocalStorage::get::<String>(&key).is_ok();
                let is_absolute = rel_path.starts_with('/') || rel_path.starts_with("Users/") || rel_path.starts_with("home/") || rel_path.starts_with("data/");
                let content_is_empty = store::load_file(&key).is_empty();
                
                if !key_exists || is_absolute || content_is_empty {
                    let file_path = if rel_path.starts_with('/') {
                        rel_path.clone()
                    } else if rel_path.starts_with("Users/") || rel_path.starts_with("home/") || rel_path.starts_with("data/") {
                        format!("/{}", rel_path)
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
                    .and_then(|d| d.query_selector(".code-editor").ok().flatten())
                {
                    if let Ok(target) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                        let text = code_sig_clone.get_untracked();
                        let index = pos_to_index(&text, line, character);
                        let _ = target.focus();
                        let _ = target.set_selection_range(index, index);
                        
                        if let Some(mirror) = web_sys::window().unwrap().document().unwrap().get_element_by_id("cursor-mirror") {
                            let text_before = &text[..index as usize];
                            mirror.set_text_content(Some(text_before));
                            let span = web_sys::window().unwrap().document().unwrap().create_element("span").unwrap();
                            span.set_text_content(Some("|"));
                            let _ = mirror.append_child(&span);
                            let span_el = span.dyn_into::<web_sys::HtmlElement>().unwrap();
                            cursor_coords_clone.set((span_el.offset_left() as f64, span_el.offset_top() as f64 + 20.0));
                        }
                        check_error_clone.run((line, character));
                    }
                }
            });
        }
    });

    let run_code = Callback::new({
        let pid = pid.clone();
        let ppath = ppath.clone();
        let plang = project.language.clone();
        move |_: ()| {
            if is_running.get_untracked() { return; }
            save_current.run(());
            let current_code = code.get_untracked();
            let lang = plang.clone();
            let path = ppath.clone();
            let pid2 = pid.clone();

            is_running.set(true);
            output.set("Compiling and running...".to_string());
            is_error.set(false);

            let cargo_toml = if lang == "rust" {
                let k = store::file_key(&pid2, "Cargo.toml");
                let v = store::load_file(&k);
                if v.is_empty() { None } else { Some(v) }
            } else { None };

            spawn_local(async move {
                let res = api::run_code(&current_code, &lang, &path, cargo_toml.as_deref()).await;
                match res {
                    Ok(r) => {
                        let mut out = r.output.clone();
                        if !r.error.is_empty() {
                            if !out.is_empty() { out.push('\n'); }
                            out.push_str(&r.error);
                        }
                        if out.is_empty() { out = "Code executed with no output.".to_string(); }
                        output.set(out);
                        is_error.set(!r.error.is_empty());
                        current_pid.set(r.pid);
                        if let Some(url) = r.url {
                            preview_url.set(Some(url));
                        }
                    }
                    Err(e) => {
                        output.set(format!("❌ Error: Could not connect to API.\n{e}"));
                        is_error.set(true);
                    }
                }
                is_running.set(false);
            });
        }
    });

    let stop_code = Callback::new(move |_: ()| {
        if let Some(pid_val) = current_pid.get_untracked() {
            spawn_local(async move {
                let _ = api::stop_process(pid_val).await;
                output.update(|o| o.push_str("\n\n[Stopped by User]"));
                current_pid.set(None);
                preview_url.set(None);
                bottom_tab.set(0);
            });
        }
    });

    let format_code = Callback::new({
        let ppath = ppath.clone();
        let plang = project.language.clone();
        let trigger_diag = trigger_diagnostics.clone();
        move |_: ()| {
            if is_running.get_untracked() { return; }
            let current_code = code.get_untracked();
            if current_code.trim().is_empty() { return; }
            
            let lang = if let Some(ref filename) = active_tab.get_untracked() {
                file_to_lsp_lang(filename)
            } else {
                plang.clone()
            };
            
            let path = ppath.clone();
            
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
        }
    });

    let add_dep = Callback::new({
        let pid = pid.clone();
        let ppath = ppath.clone();
        let plang = project.language.clone();
        let open_file = open_file.clone();
        let file_tree_data = file_tree_data.clone();
        move |_: ()| {
            let pkg = dep_input.get_untracked();
            if pkg.trim().is_empty() { return; }
            let path = ppath.clone();
            let lang = plang.clone();
            let pid_clone = pid.clone();
            let open_file_clone = open_file.clone();
            let file_tree_data_clone = file_tree_data.clone();
            dep_output.set(format!("Installing {}...", pkg));
            spawn_local(async move {
                match api::add_package(&pkg, &lang, &path).await {
                    Ok(r) => {
                        dep_output.set(if r.error.is_empty() { r.output } else { r.error });
                        if let (Some(filename), Some(content)) = (r.dependency_file_name, r.dependency_file_content) {
                            let key = store::file_key(&pid_clone, &filename);
                            store::save_file(&key, &content);
                            file_tree_data_clone.set(build_file_tree(&pid_clone));
                            open_file_clone.run(filename);
                        }
                    }
                    Err(e) => dep_output.set(format!("Error: {e}")),
                }
            });
        }
    });

    let copy_code = Callback::new({
        let show_snack = show_snack.clone();
        move |_: ()| {
            let c = code.get_untracked();
            if let Some(window) = web_sys::window() {
                let _ = window.navigator().clipboard().write_text(&c);
                show_snack.run("Code copied!".to_string());
            }
        }
    });

    let on_select = Callback::new(move |item: api::CompletionItem| {
        let cpos = cursor_pos.get_untracked();
        use wasm_bindgen::JsCast;
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Ok(Some(target)) = doc.query_selector(".code-editor") {
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
                        if i == 0 { word_start = 0; }
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
                            if let Ok(Some(target)) = doc.query_selector(".code-editor") {
                                if let Ok(target) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
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
    });

    let copied_item: RwSignal<Option<FileEntry>> = RwSignal::new(None);
    let sidebar_open: RwSignal<bool> = RwSignal::new(false);
    let sidebar_mode: RwSignal<usize> = RwSignal::new(0); // 0=files, 1=search

    let create_file = Callback::new({
        let pid = pid.clone();
        let ppath = ppath.clone();
        let show_snack = show_snack.clone();
        let open_file = open_file.clone();
        let file_tree_data = file_tree_data.clone();
        move |name: String| {
            let key = store::file_key(&pid, &name);
            store::save_file(&key, "// Start coding here...\n");
            
            // Sync to backend
            let full_path = format!("{}/{}", ppath, name);
            spawn_local(async move {
                let _ = api::save_file_api(&full_path, "// Start coding here...\n").await;
            });

            // Refresh tree
            file_tree_data.set(build_file_tree(&pid));
            show_snack.run(format!("Created file: {}", name));
            open_file.run(name);
        }
    });

    let create_folder = Callback::new({
        let pid = pid.clone();
        let ppath = ppath.clone();
        let show_snack = show_snack.clone();
        let file_tree_data = file_tree_data.clone();
        move |name: String| {
            let key = format!("codedroid_file_{}_{}/.codedroid_dir", pid, name);
            store::save_file(&key, "");

            // Sync to backend
            let full_path = format!("{}/{}", ppath, name);
            spawn_local(async move {
                let _ = api::create_dir_api(&full_path).await;
            });

            // Refresh tree
            file_tree_data.set(build_file_tree(&pid));
            show_snack.run(format!("Created folder: {}", name));
        }
    });

    let delete_entry = Callback::new({
        let pid = pid.clone();
        let ppath = ppath.clone();
        let show_snack = show_snack.clone();
        let close_tab = close_tab.clone();
        let file_tree_data = file_tree_data.clone();
        move |entry: FileEntry| {
            let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
            
            if entry.is_dir {
                // Delete all keys in LocalStorage matching prefix
                let len = storage.length().unwrap_or(0);
                let dir_prefix = format!("codedroid_file_{}_{}/", pid, entry.name);
                let placeholder_key = format!("codedroid_file_{}_{}/.codedroid_dir", pid, entry.name);
                
                let mut keys_to_remove = Vec::new();
                for i in 0..len {
                    if let Ok(Some(k)) = storage.key(i) {
                        if k.starts_with(&dir_prefix) || k == placeholder_key {
                            keys_to_remove.push(k.clone());
                            // Also close tab for any files in that directory
                            if let Some(rel) = k.strip_prefix(&format!("codedroid_file_{}_", pid)) {
                                close_tab.run(rel.to_string());
                            }
                        }
                    }
                }
                for k in keys_to_remove {
                    let _ = storage.remove_item(&k);
                }

                // Sync to backend
                let full_path = format!("{}/{}", ppath, entry.name);
                spawn_local(async move {
                    let _ = api::delete_file_api(&full_path, true).await;
                });
                show_snack.run(format!("Deleted folder: {}", entry.name));
            } else {
                // Remove single file key
                let key = store::file_key(&pid, &entry.name);
                let _ = storage.remove_item(&key);
                close_tab.run(entry.name.clone());

                // Sync to backend
                let full_path = format!("{}/{}", ppath, entry.name);
                spawn_local(async move {
                    let _ = api::delete_file_api(&full_path, false).await;
                });
                show_snack.run(format!("Deleted file: {}", entry.name));
            }

            // Refresh tree
            file_tree_data.set(build_file_tree(&pid));
        }
    });

    let copy_entry = Callback::new({
        let show_snack = show_snack.clone();
        move |entry: FileEntry| {
            copied_item.set(Some(entry.clone()));
            show_snack.run(format!("Copied {}! Long-press folder/explorer to paste.", entry.name));
        }
    });

    let paste_entry = Callback::new({
        let pid = pid.clone();
        let ppath = ppath.clone();
        let show_snack = show_snack.clone();
        let open_file = open_file.clone();
        let file_tree_data = file_tree_data.clone();
        move |target_dir: Option<String>| {
            if let Some(src_item) = copied_item.get_untracked() {
                let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
                let target_folder = target_dir.unwrap_or_default();
                
                // Determine new path
                let item_name = src_item.name.split('/').last().unwrap_or(&src_item.name);
                let mut dest_name = if target_folder.is_empty() {
                    item_name.to_string()
                } else {
                    format!("{}/{}", target_folder, item_name)
                };

                // Handle duplicates
                if dest_name == src_item.name {
                    if src_item.is_dir {
                        dest_name = format!("{}_copy", dest_name);
                    } else {
                        if let Some(idx) = dest_name.rfind('.') {
                            let (base, ext) = dest_name.split_at(idx);
                            dest_name = format!("{}_copy{}", base, ext);
                        } else {
                            dest_name = format!("{}_copy", dest_name);
                        }
                    }
                }

                if src_item.is_dir {
                    let len = storage.length().unwrap_or(0);
                    let src_prefix = format!("codedroid_file_{}_{}/", pid, src_item.name);
                    let mut copied_keys = Vec::new();
                    
                    for i in 0..len {
                        if let Ok(Some(k)) = storage.key(i) {
                            if k.starts_with(&src_prefix) {
                                if let Some(sub) = k.strip_prefix(&src_prefix) {
                                    if let Ok(Some(val)) = storage.get_item(&k) {
                                        let new_k = format!("codedroid_file_{}_{}/{}", pid, dest_name, sub);
                                        copied_keys.push((new_k, val));
                                    }
                                }
                            }
                        }
                    }

                    for (k, v) in copied_keys {
                        let _ = storage.set_item(&k, &v);
                    }

                    let src_marker = format!("codedroid_file_{}_{}/.codedroid_dir", pid, src_item.name);
                    let dest_marker = format!("codedroid_file_{}_{}/.codedroid_dir", pid, dest_name);
                    if let Ok(Some(_)) = storage.get_item(&src_marker) {
                        let _ = storage.set_item(&dest_marker, "");
                    }

                    // Sync to backend
                    let src_full = format!("{}/{}", ppath, src_item.name);
                    let dest_full = format!("{}/{}", ppath, dest_name);
                    spawn_local(async move {
                        let _ = api::copy_file_api(&src_full, &dest_full, true).await;
                    });
                    show_snack.run(format!("Pasted folder as: {}", dest_name));
                } else {
                    let src_key = store::file_key(&pid, &src_item.name);
                    if let Ok(Some(content)) = storage.get_item(&src_key) {
                        let dest_key = store::file_key(&pid, &dest_name);
                        let _ = storage.set_item(&dest_key, &content);

                        // Sync to backend
                        let src_full = format!("{}/{}", ppath, src_item.name);
                        let dest_full = format!("{}/{}", ppath, dest_name);
                        let open_file = open_file.clone();
                        let dest_name_clone = dest_name.clone();
                        spawn_local(async move {
                            let _ = api::copy_file_api(&src_full, &dest_full, false).await;
                            open_file.run(dest_name_clone);
                        });
                        show_snack.run(format!("Pasted file as: {}", dest_name));
                    }
                }

                // Refresh tree
                file_tree_data.set(build_file_tree(&pid));
            }
        }
    });

    let move_entry = Callback::new({
        let pid = pid.clone();
        let ppath = ppath.clone();
        let show_snack = show_snack.clone();
        let file_tree_data = file_tree_data.clone();
        let active_tab = active_tab.clone();
        let open_tabs = open_tabs.clone();
        
        move |(entry, new_name): (FileEntry, String)| {
            let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
            let old_name = entry.name.clone();
            
            if old_name == new_name || new_name.trim().is_empty() { return; }
            
            if entry.is_dir {
                // Moving a directory. Rename all keys in localstorage with matching prefix
                let len = storage.length().unwrap_or(0);
                let old_prefix = format!("codedroid_file_{}_{}/", pid, old_name);
                let new_prefix = format!("codedroid_file_{}_{}/", pid, new_name);
                let old_marker = format!("codedroid_file_{}_{}/.codedroid_dir", pid, old_name);
                let new_marker = format!("codedroid_file_{}_{}/.codedroid_dir", pid, new_name);
                
                let mut keys_to_move = Vec::new();
                for i in 0..len {
                    if let Ok(Some(k)) = storage.key(i) {
                        if k.starts_with(&old_prefix) {
                            keys_to_move.push(k.clone());
                        }
                    }
                }
                
                for k in keys_to_move {
                    if let Ok(Some(val)) = storage.get_item(&k) {
                        let sub = k.strip_prefix(&old_prefix).unwrap();
                        let new_k = format!("{}{}", new_prefix, sub);
                        let _ = storage.set_item(&new_k, &val);
                        let _ = storage.remove_item(&k);
                    }
                }
                
                if let Ok(Some(_)) = storage.get_item(&old_marker) {
                    let _ = storage.set_item(&new_marker, "");
                    let _ = storage.remove_item(&old_marker);
                }
                
                // Update open tabs
                open_tabs.update(|t| {
                    for tab in t.iter_mut() {
                        if tab.starts_with(&format!("{}/", old_name)) {
                            if let Some(sub) = tab.strip_prefix(&format!("{}/", old_name)) {
                                *tab = format!("{}/{}", new_name, sub);
                            }
                        }
                    }
                });
                
                // If active tab has changed name, update active_tab
                if let Some(active) = active_tab.get_untracked() {
                    if active.starts_with(&format!("{}/", old_name)) {
                        if let Some(sub) = active.strip_prefix(&format!("{}/", old_name)) {
                            active_tab.set(Some(format!("{}/{}", new_name, sub)));
                        }
                    }
                }
                
                // Sync to backend
                let src_full = format!("{}/{}", ppath, old_name);
                let dest_full = format!("{}/{}", ppath, new_name);
                spawn_local(async move {
                    let _ = api::move_file_api(&src_full, &dest_full).await;
                });
                
                show_snack.run(format!("Moved folder to: {}", new_name));
            } else {
                // Moving a single file
                let old_key = store::file_key(&pid, &old_name);
                let new_key = store::file_key(&pid, &new_name);
                
                if let Ok(Some(content)) = storage.get_item(&old_key) {
                    let _ = storage.set_item(&new_key, &content);
                    let _ = storage.remove_item(&old_key);
                }
                
                // Update open tabs
                open_tabs.update(|t| {
                    for tab in t.iter_mut() {
                        if *tab == old_name {
                            *tab = new_name.clone();
                        }
                    }
                });
                
                // Update active tab
                if active_tab.get_untracked().as_deref() == Some(&old_name) {
                    active_tab.set(Some(new_name.clone()));
                }
                
                // Sync to backend
                let src_full = format!("{}/{}", ppath, old_name);
                let dest_full = format!("{}/{}", ppath, new_name);
                spawn_local(async move {
                    let _ = api::move_file_api(&src_full, &dest_full).await;
                });
                
                show_snack.run(format!("Moved file to: {}", new_name));
            }
            
            // Refresh tree
            file_tree_data.set(build_file_tree(&pid));
        }
    });

    // Sync all files from localStorage to backend filesystem on mount
    let sync_pid = project.id.clone();
    let sync_ppath = project.path.clone();
    Effect::new(move |_| {
        let p_id = sync_pid.clone();
        let p_path = sync_ppath.clone();
        spawn_local(async move {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let len = storage.length().unwrap_or(0);
                    let prefix = format!("codedroid_file_{}_", p_id);
                    for i in 0..len {
                        if let Ok(Some(k)) = storage.key(i) {
                            if let Some(rel) = k.strip_prefix(&prefix) {
                                if rel.ends_with("/.codedroid_dir") {
                                    let dir_name = rel.trim_end_matches("/.codedroid_dir");
                                    if !dir_name.is_empty() {
                                        let full_dir_path = format!("{}/{}", p_path, dir_name);
                                        let _ = api::create_dir_api(&full_dir_path).await;
                                    }
                                } else if !rel.is_empty() {
                                    let content = store::load_file(&k);
                                    let full_file_path = format!("{}/{}", p_path, rel);
                                    let _ = api::save_file_api(&full_file_path, &content).await;
                                }
                            }
                        }
                    }
                }
            }
        });
    });

    // Open default file on mount
    Effect::new(move |_| {
        let tree = file_tree_data.get();
        if !tree.is_empty() && active_tab.get_untracked().is_none() {
            let language = project_lang_str.get_value().to_lowercase();
            let mut best_match = None;
            
            // Priority 1: Match standard entry point for the project language
            for e in tree.iter() {
                let n = e.name.to_lowercase();
                match language.as_str() {
                    "rust" if n == "src/main.rs" || n == "main.rs" => best_match = Some(e.name.clone()),
                    "go" if n == "main.go" => best_match = Some(e.name.clone()),
                    "dart" if n == "main.dart" => best_match = Some(e.name.clone()),
                    "python" if n == "main.py" => best_match = Some(e.name.clone()),
                    "java" if n == "main.java" || n == "src/main.java" => best_match = Some(e.name.clone()),
                    "c" if n == "main.c" => best_match = Some(e.name.clone()),
                    "cpp" if n == "main.cpp" => best_match = Some(e.name.clone()),
                    "javascript" | "typescript" if n == "main.js" || n == "main.ts" || n == "index.js" || n == "index.ts" => best_match = Some(e.name.clone()),
                    _ => {}
                }
                if best_match.is_some() { break; }
            }

            // Priority 2: Match any entry point from the general list
            if best_match.is_none() {
                let main_files = [
                    "src/main.rs", "main.rs", "main.dart", "main.go", "main.py",
                    "main.js", "main.ts", "src/main.js", "src/main.ts",
                    "src/main.jsx", "src/main.tsx", "index.js", "index.ts",
                    "index.html", "Main.java", "main.c", "main.cpp",
                    "Program.cs", "main.kt", "main.swift", "main.rb",
                ];
                for e in tree.iter() {
                    let n = e.name.to_lowercase();
                    if main_files.iter().any(|&m| {
                        let m_low = m.to_lowercase();
                        n == m_low || n.ends_with(&format!("/{}", m_low))
                    }) {
                        best_match = Some(e.name.clone());
                        break;
                    }
                }
            }

            let default_file = best_match.unwrap_or_else(|| tree[0].name.clone());

            spawn_local(async move {
                // Small delay to ensure the editor and store are fully ready
                gloo_timers::future::TimeoutFuture::new(100).await;
                if active_tab.get_untracked().is_none() {
                    open_file.run(default_file);
                }
            });
        }
    });

    view! {
        <div class="editor-page-root">
            <AppBar title=project.name.clone() back=true>
                <button class="btn btn-icon btn-menu" title="Toggle Files"
                    style=move || {
                        let is_active = sidebar_open.get() && sidebar_mode.get() == 0;
                        if is_active {
                            "margin-right: 6px; background: rgba(99, 102, 241, 0.2); color: var(--accent); border-color: var(--accent);"
                        } else {
                            "margin-right: 6px;"
                        }
                    }
                    on:click=move |_| {
                        if sidebar_open.get() && sidebar_mode.get() == 0 {
                            sidebar_open.set(false);
                        } else {
                            sidebar_mode.set(0);
                            sidebar_open.set(true);
                        }
                    }>
                    <LucideIcon name="folder" size="20" />
                </button>
                <button class="btn btn-icon" title="Find in Current File (Ctrl+F)"
                    on:click=move |_| show_search.update(|v| *v = !*v)>
                    <LucideIcon name="search" size="20" />
                </button>
                <button class="btn btn-icon" title="Search and Replace Project"
                    style=move || {
                        let is_active = sidebar_open.get() && sidebar_mode.get() == 1;
                        if is_active {
                            "background: rgba(99, 102, 241, 0.2); color: var(--accent); border-color: var(--accent);"
                        } else {
                            ""
                        }
                    }
                    on:click=move |_| {
                        if sidebar_open.get() && sidebar_mode.get() == 1 {
                            sidebar_open.set(false);
                        } else {
                            sidebar_mode.set(1);
                            sidebar_open.set(true);
                        }
                    }>
                    <LucideIcon name="replace" size="20" />
                </button>
                <button class="btn btn-icon desktop-only" title="Dependencies"
                    on:click=move |_| show_deps.update(|v| *v = !*v)>
                    <LucideIcon name="package" size="20" />
                </button>
                <button class="btn btn-icon desktop-only" title="Format Code (Shift+Alt+F)"
                    on:click=move |_| format_code.run(())>
                    <LucideIcon name="code" size="20" />
                </button>
                <button class="btn btn-icon desktop-only" title="Go to Definition (F12)"
                    on:click=move |_| trigger_definition.run(())>
                    <LucideIcon name="locate-fixed" size="20" />
                </button>
                <button class="btn btn-icon desktop-only" title="Find References (Shift+F12)"
                    on:click=move |_| trigger_references.run(())>
                    <LucideIcon name="search-code" size="20" />
                </button>
                <div class="more-menu-container mobile-only">
                    <button class="btn btn-icon" title="More Options"
                        on:click=move |e: web_sys::MouseEvent| {
                            e.stop_propagation();
                            show_more_menu.update(|v| *v = !*v);
                        }>
                        <LucideIcon name="more-vertical" size="20" />
                    </button>
                    {move || show_more_menu.get().then(|| view! {
                        <>
                        <div class="more-menu-backdrop" on:click=move |e: web_sys::MouseEvent| {
                            e.stop_propagation();
                            show_more_menu.set(false);
                        } />
                        <div class="more-menu-dropdown" on:click=move |e: web_sys::MouseEvent| e.stop_propagation()>
                            <button class="more-menu-item"
                                on:click=move |_| {
                                    sidebar_mode.set(1);
                                    sidebar_open.set(true);
                                    show_more_menu.set(false);
                                }>
                                <LucideIcon name="replace" size="18" />
                                <span>"Search / Replace"</span>
                            </button>
                            <button class="more-menu-item"
                                on:click=move |_| {
                                    show_deps.update(|v| *v = !*v);
                                    show_more_menu.set(false);
                                }>
                                <LucideIcon name="package" size="18" />
                                <span>"Dependencies"</span>
                            </button>
                            <button class="more-menu-item"
                                on:click=move |_| {
                                    format_code.run(());
                                    show_more_menu.set(false);
                                }>
                                <LucideIcon name="code" size="18" />
                                <span>"Format Code"</span>
                            </button>
                            <button class="more-menu-item"
                                on:click=move |_| {
                                    trigger_definition.run(());
                                    show_more_menu.set(false);
                                }>
                                <LucideIcon name="locate-fixed" size="18" />
                                <span>"Go to Definition"</span>
                            </button>
                            <button class="more-menu-item"
                                on:click=move |_| {
                                    trigger_references.run(());
                                    show_more_menu.set(false);
                                }>
                                <LucideIcon name="search-code" size="18" />
                                <span>"Find References"</span>
                            </button>
                        </div>
                        </>
                    })}
                </div>
                {move || dirty.get().then(|| view! {
                    <button class="btn btn-icon" title="Save (Ctrl+S)"
                        on:click=move |_| save_current.run(())
                    >
                        <LucideIcon name="save" size="20" />
                    </button>
                })}
                {move || current_pid.get().map(|_| view! {
                    <button class="btn btn-danger" style="display:inline-flex; align-items:center; gap:6px;" on:click=move |_| stop_code.run(())>
                        <LucideIcon name="square" size="14" /> <span class="btn-text">"Stop"</span>
                    </button>
                })}
                <button class="btn btn-success" style="display:inline-flex; align-items:center; gap:6px;" disabled=move || is_running.get()
                    on:click=move |_| run_code.run(())
                >
                    {move || if is_running.get() {
                        view! { <><span class="spinner"></span><span class="btn-text">" Running..."</span></> }.into_any()
                    } else {
                        view! { <><LucideIcon name="play" size="14" /> <span class="btn-text">"Run"</span></> }.into_any()
                    }}
                </button>
                {move || preview_url.get().is_some().then(|| view! {
                    <button class="btn btn-success mobile-preview-toggle-btn"
                        style="display:inline-flex; align-items:center; gap:6px; background:#4f46e5; border-color:#4f46e5;"
                        on:click=move |_| show_mobile_full_preview.set(true)>
                        <LucideIcon name="eye" size="16" />
                        <span class="btn-text">"Preview"</span>
                    </button>
                })}
            </AppBar>

            <div class="editor-layout">
                {move || {
                    if sidebar_mode.get() == 0 {
                        view! {
                            <FileTree 
                                file_tree=file_tree_data.into()
                                active_tab=active_tab.into()
                                open_file=open_file
                                lang_icon=lang_icon(&project_lang_str.get_value()).to_string()
                                project_name=project.name.clone()
                                create_file=create_file
                                create_folder=create_folder
                                delete_entry=delete_entry
                                copy_entry=copy_entry
                                copied_item=copied_item.into()
                                paste_entry=paste_entry
                                move_entry=move_entry
                                sidebar_open=sidebar_open.into()
                                toggle_sidebar=Callback::new(move |_: ()| sidebar_open.set(false))
                                sidebar_mode=sidebar_mode
                            />
                        }.into_any()
                    } else {
                        view! {
                            <ProjectSearchReplacePanel
                                project_id=pid.clone()
                                project_path=ppath.clone()
                                file_tree=file_tree_data.into()
                                file_tree_data=file_tree_data
                                active_tab=active_tab
                                code=code
                                dirty=dirty
                                project_query=project_search_text
                                replace_text=project_replace_text
                                open_file=open_file
                                trigger_diagnostics=trigger_diagnostics
                                show_snack=show_snack
                                sidebar_open=sidebar_open.into()
                                close_sidebar=Callback::new(move |_: ()| sidebar_open.set(false))
                                sidebar_mode=sidebar_mode
                            />
                        }.into_any()
                    }
                }}

                <div class="editor-main">
                    <TabStrip 
                        open_tabs=open_tabs.into()
                        active_tab=active_tab.into()
                        dirty=dirty.into()
                        open_file=open_file
                        close_tab=close_tab
                    />

                    <SearchBar 
                        show_search=show_search
                        find_text=find_text
                        find_index=find_index
                        code=code
                    />

                    <EditorCodeArea
                        settings=settings
                        code=code
                        dirty=dirty
                        active_tab=active_tab
                        diagnostics_list=diagnostics_list
                        active_error=active_error
                        cursor_pos=cursor_pos
                        cursor_coords=cursor_coords
                        suggestions=suggestions
                        selected_idx=selected_idx
                        project_lang_str=project_lang_str
                        project_path_str=project_path_str
                        last_request_id=last_request_id
                        trigger_diagnostics=trigger_diagnostics
                        save_current=save_current
                        format_code=format_code
                        show_search=show_search
                        check_error_at_cursor=check_error_at_cursor
                        on_select=on_select
                        show_snack=show_snack
                        trigger_definition=trigger_definition
                        trigger_references=trigger_references
                    />

                    <BottomPanel 
                        bottom_tab=bottom_tab
                        output=output.into()
                        is_error=is_error.into()
                        show_snack=show_snack
                        diagnostics_list=diagnostics_list.into()
                        on_click_problem=on_click_problem
                        code=code
                        language=Signal::derive(move || project_lang_str.get_value())
                        references_list=references_list
                        on_click_reference=on_click_reference
                        active_tab=active_tab.into()
                    />

                    <EditorFooter
                        code=code
                        dirty=dirty
                        settings=settings
                        copy_code=copy_code
                    />
                </div>

                <PreviewPanel
                    preview_url=preview_url.into()
                    refresh_key=refresh_key
                />
            </div>

            <DependencyModal 
                show_deps=show_deps
                dep_input=dep_input
                dep_output=dep_output.into()
                add_dep=add_dep
            />

            <MobilePreviewOverlay
                show_mobile_full_preview=show_mobile_full_preview
                preview_url=preview_url.into()
                refresh_key=refresh_key
            />

            <Snackbar message=snack_msg.read_only() />
        </div>
    }.into_any()
}

fn uri_to_relative(uri: &str, project_path: &str) -> String {
    let uri_clean = uri.replace('\\', "/");
    let proj_clean = project_path.replace('\\', "/");
    
    let prefix = format!("file://{}/", proj_clean);
    let prefix_triple = format!("file:///{}", proj_clean);
    let prefix_triple_alt = format!("file://{}", proj_clean);
    
    if uri_clean.starts_with(&prefix) {
        let mut rel = uri_clean.strip_prefix(&prefix).unwrap_or(&uri_clean).to_string();
        if rel.starts_with('/') {
            rel = rel.trim_start_matches('/').to_string();
        }
        rel
    } else if uri_clean.starts_with(&prefix_triple) {
        let mut rel = uri_clean.strip_prefix(&prefix_triple).unwrap_or(&uri_clean).to_string();
        if rel.starts_with('/') {
            rel = rel.trim_start_matches('/').to_string();
        }
        rel
    } else if uri_clean.starts_with(&prefix_triple_alt) {
        let mut rel = uri_clean.strip_prefix(&prefix_triple_alt).unwrap_or(&uri_clean).to_string();
        if rel.starts_with('/') {
            rel = rel.trim_start_matches('/').to_string();
        }
        rel
    } else {
        let needle = format!("/{}", proj_clean.trim_start_matches('/'));
        if let Some(pos) = uri_clean.find(&needle) {
            let mut rel = uri_clean[pos + needle.len() + 1..].to_string();
            if rel.starts_with('/') {
                rel = rel.trim_start_matches('/').to_string();
            }
            rel
        } else {
            // Absolute path outside the project. Ensure it starts with '/'
            let path_without_scheme = uri_clean.strip_prefix("file://").unwrap_or(&uri_clean).to_string();
            if !path_without_scheme.starts_with('/') {
                format!("/{}", path_without_scheme)
            } else {
                path_without_scheme
            }
        }
    }
}
