use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use wasm_bindgen_futures::spawn_local;

pub mod utils;
pub mod components;

use utils::*;
use components::*;
use crate::models::{Project, Settings, lang_icon};
use crate::store;
use crate::api;
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
    let snack_msg: RwSignal<Option<String>> = RwSignal::new(None);
    let file_tree_data: RwSignal<Vec<FileEntry>> = RwSignal::new(build_file_tree(&project.id));
    let show_deps: RwSignal<bool> = RwSignal::new(false);
    let dep_input: RwSignal<String> = RwSignal::new(String::new());
    let dep_output: RwSignal<String> = RwSignal::new(String::new());
    let suggestions = RwSignal::new(Vec::<api::CompletionItem>::new());
    let selected_idx = RwSignal::new(0);
    let cursor_pos = RwSignal::new(0);
    let cursor_coords = RwSignal::new((0.0, 0.0));
    let last_request_id = RwSignal::new(0u64);
    let diagnostics_list = RwSignal::new(Vec::<api::Diagnostic>::new());
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
            spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(800).await;
                if last_diag_req.get_untracked() == req_id {
                    if let Ok(resp) = api::get_diagnostics_api(&code_val, &file_lang, &ppath).await {
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
        move |(line, _col): (u32, u32)| {
            let diags = diagnostics_list.get_untracked();
            let diag_opt = diags.iter().find(|d| d.range.start.line == line).cloned();
            
            if let Some(diag) = diag_opt {
                let current = active_error.get_untracked();
                if let Some((curr_diag, _, _)) = &current {
                    if curr_diag.message == diag.message && curr_diag.range.start.line == diag.range.start.line {
                        return;
                    }
                }
                
                active_error.set(Some((diag.clone(), Vec::new(), true)));
                
                let code_val = code_sig.get_untracked();
                let lang_val = project_lang_str.get_value();
                
                spawn_local(async move {
                    if let Ok(resp) = api::get_error_suggestions_api(&code_val, &lang_val, &diag).await {
                        active_error.set(Some((diag, resp.suggestions, false)));
                    } else {
                        active_error.set(None);
                    }
                });
            } else {
                active_error.set(None);
            }
        }
    });

    let on_click_problem = Callback::new({
        let code_signal = code;
        let check_error = check_error_at_cursor.clone();
        let cursor_coords = cursor_coords.clone();
        move |(line, character): (u32, u32)| {
            use wasm_bindgen::JsCast;
            if let Some(target) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.query_selector(".code-editor").ok().flatten())
            {
                if let Ok(target) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                    let text = code_signal.get_untracked();
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
                        cursor_coords.set((span_el.offset_left() as f64, span_el.offset_top() as f64 + 20.0));
                    }
                    
                    check_error.run((line, character));
                }
            }
        }
    });

    let pid = project.id.clone();
    let open_file = Callback::new({
        let pid = pid.clone();
        let trigger_diag = trigger_diagnostics.clone();
        move |name: String| {
            let key = store::file_key(&pid, &name);
            let content = store::load_file(&key);
            open_tabs.update(|t| { if !t.contains(&name) { t.push(name.clone()); }});
            active_tab.set(Some(name));
            code.set(content.clone());
            dirty.set(false);
            trigger_diag.run(content);
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
        let ppath = ppath.clone();
        let plang = project.language.clone();
        move |_: ()| {
            let pkg = dep_input.get_untracked();
            if pkg.trim().is_empty() { return; }
            let path = ppath.clone();
            let lang = plang.clone();
            dep_output.set(format!("Installing {}...", pkg));
            spawn_local(async move {
                match api::add_package(&pkg, &lang, &path).await {
                    Ok(r) => dep_output.set(if r.error.is_empty() { r.output } else { r.error }),
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
                    style="margin-right: 6px;"
                    on:click=move |_| sidebar_open.update(|v| *v = !*v)>
                    <LucideIcon name="folder" size="20" />
                </button>
                <button class="btn btn-icon" title="Search (Ctrl+F)"
                    on:click=move |_| show_search.update(|v| *v = !*v)>
                    <LucideIcon name="search" size="20" />
                </button>
                <button class="btn btn-icon" title="Dependencies"
                    on:click=move |_| show_deps.update(|v| *v = !*v)>
                    <LucideIcon name="package" size="20" />
                </button>
                <button class="btn btn-icon" title="Format Code (Shift+Alt+F)"
                    on:click=move |_| format_code.run(())>
                    <LucideIcon name="code" size="20" />
                </button>
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
                />

                <div class="editor-main">
                    <TabStrip 
                        open_tabs=open_tabs.into()
                        active_tab=active_tab.into()
                        dirty=dirty.into()
                        open_file=open_file
                        close_tab=close_tab
                    />

                    {move || show_search.get().then(|| view! {
                        <div class="search-bar">
                            <input class="input" type="text" placeholder="Find..."
                                prop:value=move || find_text.get()
                                on:input=move |e| find_text.set(event_target_value(&e))
                            />
                            <button class="btn btn-primary" style="padding:6px 12px;font-size:12px">"Find Next"</button>
                            <button class="btn btn-icon" on:click=move |_| show_search.set(false)>
                                <LucideIcon name="x" size="16" />
                            </button>
                        </div>
                    })}

                    <div class="code-area" style="flex:2">
                        {move || {
                            let s = settings.get();
                            let content = code.get();
                            let ext = active_tab.get().map(|n| file_extension(&n).to_string()).unwrap_or_default();
                            let highlighted_lines = highlight_code_lines(&content, &ext);
                            let diags = diagnostics_list.get();

                            let container_class = if s.show_line_numbers {
                                "code-container"
                            } else {
                                "code-container hide-line-numbers"
                            };

                            view! {
                                <div class=container_class style=move || format!(
                                        "font-size:{}px;white-space:{};tab-size:{}",
                                        s.font_size,
                                        if s.word_wrap { "pre-wrap" } else { "pre" },
                                        s.tab_size,
                                    )>
                                    <div class="code-layer code-highlight">
                                        {highlighted_lines.into_iter().enumerate().map(|(idx, html_line)| {
                                            let n = idx + 1;
                                            let has_error = diags.iter().any(|d| d.range.start.line == (n - 1) as u32 && d.severity.unwrap_or(1) == 1);
                                            let has_warning = diags.iter().any(|d| d.range.start.line == (n - 1) as u32 && d.severity.unwrap_or(1) == 2);
                                            
                                            let gutter_class = if has_error {
                                                "line-number-item has-error"
                                            } else if has_warning {
                                                "line-number-item has-warning"
                                            } else {
                                                "line-number-item"
                                            };
                                            
                                            let gutter_marker = if has_error {
                                                "🔴"
                                            } else if has_warning {
                                                "🟡"
                                            } else {
                                                ""
                                            };

                                            view! {
                                                <div class="editor-line">
                                                    {s.show_line_numbers.then(|| {
                                                        view! {
                                                            <div class="line-number-gutter">
                                                                <div class=gutter_class title=move || if has_error { "Error on this line" } else if has_warning { "Warning on this line" } else { "" }>
                                                                    {(!gutter_marker.is_empty()).then(|| {
                                                                        view! { <span class="gutter-error-icon">{gutter_marker}</span> }
                                                                    })}
                                                                    <span class="gutter-number-text">{n}</span>
                                                                </div>
                                                            </div>
                                                        }
                                                    })}
                                                    <div class="line-content" inner_html=html_line></div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                    <textarea
                                        class="code-layer code-editor"
                                        spellcheck="false"
                                        prop:value=move || code.get()
                                        on:beforeinput=move |input_ev: web_sys::InputEvent| {
                                            use wasm_bindgen::JsCast;
                                            if let Some(data) = input_ev.data() {
                                                if data.chars().count() == 1 {
                                                    let ch = data.chars().next().unwrap();
                                                    let key = ch.to_string();
                                                    if key == "(" || key == "{" || key == "[" || key == "\"" || key == "'" {
                                                        input_ev.prevent_default();
                                                        let target = input_ev.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                                        let start = target.selection_start().unwrap().unwrap_or(0);
                                                        let end = target.selection_end().unwrap().unwrap_or(0);
                                                        let val = js_sys::JsString::from(target.value());
                                                        
                                                        let close_char = match key.as_str() {
                                                            "(" => ")",
                                                            "{" => "}",
                                                            "[" => "]",
                                                            "\"" => "\"",
                                                            "'" => "'",
                                                            _ => "",
                                                        };
                                                        
                                                        if start != end {
                                                            let selected_text = val.substring(start, end);
                                                            let new_val = format!(
                                                                "{}{}{}{}{}",
                                                                String::from(val.substring(0, start)),
                                                                key,
                                                                String::from(selected_text),
                                                                close_char,
                                                                String::from(val.substring(end, val.length()))
                                                            );
                                                            code.set(new_val);
                                                            dirty.set(true);
                                                            let new_start = start + 1;
                                                            let new_end = end + 1;
                                                            spawn_local(async move {
                                                                let _ = gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                                                                let _ = target.set_selection_range(new_start, new_end);
                                                            });
                                                        } else {
                                                            let new_val = format!(
                                                                "{}{}{}{}",
                                                                String::from(val.substring(0, start)),
                                                                key,
                                                                close_char,
                                                                String::from(val.substring(end, val.length()))
                                                            );
                                                            code.set(new_val);
                                                            dirty.set(true);
                                                            let new_pos = start + 1;
                                                            spawn_local(async move {
                                                                let _ = gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                                                                let _ = target.set_selection_range(new_pos, new_pos);
                                                            });
                                                        }
                                                    }
                                                    else if key == ")" || key == "}" || key == "]" || key == "\"" || key == "'" {
                                                        let target = input_ev.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                                        let start = target.selection_start().unwrap().unwrap_or(0);
                                                        let end = target.selection_end().unwrap().unwrap_or(0);
                                                        if start == end {
                                                            let val = js_sys::JsString::from(target.value());
                                                            if start < val.length() {
                                                                let next_char = val.substring(start, start + 1);
                                                                if next_char == key {
                                                                    input_ev.prevent_default();
                                                                    let new_pos = start + 1;
                                                                    let _ = target.set_selection_range(new_pos, new_pos);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        on:input=move |e: web_sys::Event| {
                                            use wasm_bindgen::JsCast;
                                            let target = e.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                            let val = target.value();
                                            code.set(val.clone());
                                            dirty.set(true);
                                            active_error.set(None);
                                             trigger_diagnostics.run(val.clone());
                                            if settings.get_untracked().auto_save { save_current.run(()); }

                                            let start = target.selection_start().unwrap().unwrap_or(0);
                                            cursor_pos.set(start);
                                            
                                            if let Some(mirror) = web_sys::window().unwrap().document().unwrap().get_element_by_id("cursor-mirror") {
                                                let text_before = &val[..start as usize];
                                                mirror.set_text_content(Some(text_before));
                                                let span = web_sys::window().unwrap().document().unwrap().create_element("span").unwrap();
                                                span.set_text_content(Some("|"));
                                                let _ = mirror.append_child(&span);
                                                let span_el = span.dyn_into::<web_sys::HtmlElement>().unwrap();
                                                cursor_coords.set((span_el.offset_left() as f64, span_el.offset_top() as f64 + 20.0));
                                            }

                                            let (line, character) = {
                                                let text_before = &val[..start as usize];
                                                let lines: Vec<&str> = text_before.split('\n').collect();
                                                (lines.len().saturating_sub(1) as u32, lines.last().map(|l| l.chars().count()).unwrap_or(0) as u32)
                                            };
                                            selected_idx.set(0);

                                            let is_source = if let Some(ref filename) = active_tab.get_untracked() {
                                                is_project_source_file(filename, &project_lang_str.get_value())
                                            } else {
                                                false
                                            };

                                            let chars: Vec<char> = val.chars().collect();
                                            if is_source && start > 0 && start as usize <= chars.len() {
                                                let last_char = chars[(start - 1) as usize];
                                                if last_char.is_alphanumeric() || last_char == '.' || last_char == '<' || last_char == '/' || last_char == ':' || last_char == '@' || last_char == '$' || last_char == '-' || last_char == '"' || last_char == '\'' || last_char == '=' {
                                                    let active_file = active_tab.get_untracked().unwrap_or_default();
                                                    let lang = file_to_lsp_lang(&active_file);
                                                    let path = project_path_str.get_value();
                                                    let req_id = last_request_id.get_untracked() + 1;
                                                    last_request_id.set(req_id);
                                                    spawn_local(async move {
                                                        gloo_timers::future::TimeoutFuture::new(150).await;
                                                        if last_request_id.get_untracked() == req_id {
                                                            if let Ok(resp) = api::get_completions_api(&val, &lang, &path, line, character).await {
                                                                if last_request_id.get_untracked() == req_id { suggestions.set(resp.suggestions); }
                                                            }
                                                        }
                                                    });
                                                } else { suggestions.set(Vec::new()); }
                                            } else { suggestions.set(Vec::new()); }
                                        }
                                        on:keydown=move |e: web_sys::KeyboardEvent| {
                                            if (e.ctrl_key() || e.meta_key()) && e.key() == "s" { e.prevent_default(); save_current.run(()); }
                                            if e.shift_key() && e.alt_key() && (e.key() == "f" || e.key() == "F") { e.prevent_default(); format_code.run(()); }
                                            if (e.ctrl_key() || e.meta_key()) && e.key() == "f" { e.prevent_default(); show_search.update(|v| *v = !*v); }
                                            if !suggestions.get().is_empty() {
                                                let current = selected_idx.get();
                                                let total = suggestions.get().len();
                                                match e.key().as_str() {
                                                    "ArrowDown" => { e.prevent_default(); selected_idx.set((current + 1) % total); }
                                                    "ArrowUp" => { e.prevent_default(); selected_idx.set((current + total - 1) % total); }
                                                    "Enter" | "Tab" => { e.prevent_default(); if let Some(s) = suggestions.get().get(current) { on_select.run(s.clone()); } }
                                                    "Escape" => { suggestions.set(Vec::new()); }
                                                    _ => {}
                                                }
                                                return;
                                            }
                                            if e.ctrl_key() && e.key() == " " {
                                                e.prevent_default();
                                                let is_source = if let Some(ref filename) = active_tab.get_untracked() {
                                                    is_project_source_file(filename, &project_lang_str.get_value())
                                                } else {
                                                    false
                                                };
                                                if !is_source {
                                                    return;
                                                }
                                                use wasm_bindgen::JsCast;
                                                let target = e.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                                let val = target.value();
                                                let start = target.selection_start().unwrap().unwrap_or(0);
                                                let active_file = active_tab.get_untracked().unwrap_or_default();
                                                let lang = file_to_lsp_lang(&active_file);
                                                let path = project_path_str.get_value();
                                                let before_cursor = val.chars().take(start as usize).collect::<String>();
                                                let lines: Vec<&str> = before_cursor.split('\n').collect();
                                                let line = lines.len().saturating_sub(1) as u32;
                                                let character = lines.last().map(|l| l.chars().count()).unwrap_or(0) as u32;
                                                spawn_local(async move {
                                                    if let Ok(resp) = api::get_completions_api(&val, &lang, &path, line, character).await {
                                                        suggestions.set(resp.suggestions);
                                                    }
                                                });
                                            }
                                            if e.key() == "Tab" {
                                                e.prevent_default();
                                                let spaces = " ".repeat(settings.get_untracked().tab_size);
                                                use wasm_bindgen::JsCast;
                                                let target = e.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                                let start = target.selection_start().unwrap().unwrap_or(0);
                                                let end = target.selection_end().unwrap().unwrap_or(0);
                                                let val = js_sys::JsString::from(target.value());
                                                let new_val = format!("{}{}{}", String::from(val.substring(0, start)), spaces, String::from(val.substring(end, val.length())));
                                                code.set(new_val);
                                                dirty.set(true);
                                                let new_pos = start + spaces.len() as u32;
                                                spawn_local(async move {
                                                    let _ = gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                                                    let _ = target.set_selection_range(new_pos, new_pos);
                                                });
                                            }
                                            
                                            let key = e.key();
                                            if key == "(" || key == "{" || key == "[" || key == "\"" || key == "'" {
                                                e.prevent_default();
                                                use wasm_bindgen::JsCast;
                                                let target = e.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                                let start = target.selection_start().unwrap().unwrap_or(0);
                                                let end = target.selection_end().unwrap().unwrap_or(0);
                                                let val = js_sys::JsString::from(target.value());
                                                
                                                let close_char = match key.as_str() {
                                                    "(" => ")",
                                                    "{" => "}",
                                                    "[" => "]",
                                                    "\"" => "\"",
                                                    "'" => "'",
                                                    _ => "",
                                                };
                                                
                                                if start != end {
                                                    let selected_text = val.substring(start, end);
                                                    let new_val = format!(
                                                        "{}{}{}{}{}",
                                                        String::from(val.substring(0, start)),
                                                        key,
                                                        String::from(selected_text),
                                                        close_char,
                                                        String::from(val.substring(end, val.length()))
                                                    );
                                                    code.set(new_val);
                                                    dirty.set(true);
                                                    let new_start = start + 1;
                                                    let new_end = end + 1;
                                                    spawn_local(async move {
                                                        let _ = gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                                                        let _ = target.set_selection_range(new_start, new_end);
                                                    });
                                                } else {
                                                    let new_val = format!(
                                                        "{}{}{}{}",
                                                        String::from(val.substring(0, start)),
                                                        key,
                                                        close_char,
                                                        String::from(val.substring(end, val.length()))
                                                    );
                                                    code.set(new_val);
                                                    dirty.set(true);
                                                    let new_pos = start + 1;
                                                    spawn_local(async move {
                                                        let _ = gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                                                        let _ = target.set_selection_range(new_pos, new_pos);
                                                    });
                                                }
                                            }
                                            else if key == ")" || key == "}" || key == "]" || key == "\"" || key == "'" {
                                                use wasm_bindgen::JsCast;
                                                let target = e.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                                let start = target.selection_start().unwrap().unwrap_or(0);
                                                let end = target.selection_end().unwrap().unwrap_or(0);
                                                if start == end {
                                                    let val = js_sys::JsString::from(target.value());
                                                    if start < val.length() {
                                                        let next_char = val.substring(start, start + 1);
                                                        if next_char == key {
                                                            e.prevent_default();
                                                            let new_pos = start + 1;
                                                            let _ = target.set_selection_range(new_pos, new_pos);
                                                        }
                                                    }
                                                }
                                            }
                                            else if key == "Backspace" {
                                                use wasm_bindgen::JsCast;
                                                let target = e.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                                let start = target.selection_start().unwrap().unwrap_or(0);
                                                let end = target.selection_end().unwrap().unwrap_or(0);
                                                if start == end && start > 0 {
                                                    let val = js_sys::JsString::from(target.value());
                                                    if start < val.length() {
                                                        let prev_char = val.substring(start - 1, start);
                                                        let next_char = val.substring(start, start + 1);
                                                        
                                                        let is_pair = match (String::from(prev_char).as_str(), String::from(next_char).as_str()) {
                                                            ("(", ")") => true,
                                                            ("{", "}") => true,
                                                            ("[", "]") => true,
                                                            ("\"", "\"") => true,
                                                            ("'", "'") => true,
                                                            _ => false,
                                                        };
                                                        
                                                        if is_pair {
                                                            e.prevent_default();
                                                            let new_val = format!(
                                                                "{}{}",
                                                                String::from(val.substring(0, start - 1)),
                                                                String::from(val.substring(start + 1, val.length()))
                                                            );
                                                            code.set(new_val);
                                                            dirty.set(true);
                                                            let new_pos = start - 1;
                                                            spawn_local(async move {
                                                                let _ = gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                                                                let _ = target.set_selection_range(new_pos, new_pos);
                                                            });
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        on:click=move |e: web_sys::MouseEvent| {
                                            suggestions.set(Vec::new());
                                            use wasm_bindgen::JsCast;
                                            let target = e.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                            let start = target.selection_start().unwrap().unwrap_or(0);
                                            cursor_pos.set(start);
                                            let val = target.value();
                                            
                                            if let Some(mirror) = web_sys::window().unwrap().document().unwrap().get_element_by_id("cursor-mirror") {
                                                let text_before = &val[..start as usize];
                                                mirror.set_text_content(Some(text_before));
                                                let span = web_sys::window().unwrap().document().unwrap().create_element("span").unwrap();
                                                span.set_text_content(Some("|"));
                                                let _ = mirror.append_child(&span);
                                                let span_el = span.dyn_into::<web_sys::HtmlElement>().unwrap();
                                                cursor_coords.set((span_el.offset_left() as f64, span_el.offset_top() as f64 + 20.0));
                                            }

                                            let (line, character) = {
                                                let text_before = &val[..start as usize];
                                                let lines: Vec<&str> = text_before.split('\n').collect();
                                                (lines.len().saturating_sub(1) as u32, lines.last().map(|l| l.chars().count()).unwrap_or(0) as u32)
                                            };
                                            check_error_at_cursor.run((line, character));
                                        }
                                        on:keyup=move |e: web_sys::KeyboardEvent| {
                                            let key = e.key();
                                            let is_nav = ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Home", "End", "PageUp", "PageDown"].contains(&key.as_str());
                                            if is_nav {
                                                if ("ArrowUp" == key || "ArrowDown" == key) && !suggestions.get().is_empty() {
                                                    return;
                                                }
                                                if ["ArrowLeft", "ArrowRight", "Home", "End", "PageUp", "PageDown"].contains(&key.as_str()) {
                                                    suggestions.set(Vec::new());
                                                }
                                                use wasm_bindgen::JsCast;
                                                let target = e.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                                let start = target.selection_start().unwrap().unwrap_or(0);
                                                cursor_pos.set(start);
                                                let val = target.value();
                                                
                                                if let Some(mirror) = web_sys::window().unwrap().document().unwrap().get_element_by_id("cursor-mirror") {
                                                    let text_before = &val[..start as usize];
                                                    mirror.set_text_content(Some(text_before));
                                                    let span = web_sys::window().unwrap().document().unwrap().create_element("span").unwrap();
                                                    span.set_text_content(Some("|"));
                                                    let _ = mirror.append_child(&span);
                                                    let span_el = span.dyn_into::<web_sys::HtmlElement>().unwrap();
                                                    cursor_coords.set((span_el.offset_left() as f64, span_el.offset_top() as f64 + 20.0));
                                                }

                                                let (line, character) = {
                                                    let text_before = &val[..start as usize];
                                                    let lines: Vec<&str> = text_before.split('\n').collect();
                                                    (lines.len().saturating_sub(1) as u32, lines.last().map(|l| l.chars().count()).unwrap_or(0) as u32)
                                                };
                                                check_error_at_cursor.run((line, character));
                                            }
                                        }
                                    />
                                    {move || (!suggestions.get().is_empty()).then(|| {
                                        let coords = cursor_coords.get();
                                        let items = suggestions.get();
                                        let selected = selected_idx.get();
                                        let current_item = items.get(selected).cloned();
                                        view! {
                                            <div class="suggestions-floating" style=format!("left:{}px; top:{}px", coords.0, coords.1)>
                                                {move || suggestions.get().into_iter().enumerate().map(|(i, s)| {
                                                    let s2 = s.clone();
                                                    let s3 = s.clone();
                                                    view! {
                                                        <button 
                                                            class=move || if selected_idx.get() == i { "suggestion-item selected" } else { "suggestion-item" }
                                                            on:mousedown=move |e: web_sys::MouseEvent| { e.prevent_default(); on_select.run(s2.clone()); }
                                                            on:touchstart=move |e: web_sys::TouchEvent| { e.prevent_default(); on_select.run(s3.clone()); }
                                                            on:mouseenter=move |_| selected_idx.set(i)
                                                        >
                                                            <span class="suggestion-kind">{kind_icon(s.kind)}</span>
                                                            <span class="suggestion-label">{s.label.clone()}</span>
                                                            {s.detail.map(|d| view! { <span class="suggestion-detail">{d}</span> })}
                                                        </button>
                                                    }
                                                }).collect_view()}
                                                {move || current_item.as_ref().and_then(|item| item.documentation.as_ref()).map(|docs| view! {
                                                    <div class="suggestion-docs">{docs.clone()}</div>
                                                })}
                                            </div>
                                        }
                                    })}
                                    {move || {
                                        if !suggestions.get().is_empty() {
                                            return view! { "" }.into_any();
                                        }
                                        if let Some((diag, suggs, loading)) = active_error.get() {
                                            let coords = cursor_coords.get();
                                            let snack = show_snack;
                                            let code_sig = code;
                                            let active_error_sig = active_error;
                                            
                                            view! {
                                                <div class="error-floating-popover" style=format!("left:{}px; top:{}px", coords.0, coords.1)>
                                                    <div class="error-floating-header">
                                                        <span class="error-floating-icon">"🔴"</span>
                                                        <span class="error-floating-title">{diag.message}</span>
                                                    </div>
                                                    
                                                    {move || {
                                                        if loading {
                                                            view! {
                                                                <div class="error-floating-loading">
                                                                    <div class="spinner" style="width:12px;height:12px;border-width:1.5px;display:inline-block;vertical-align:middle;margin-right:6px" />
                                                                    "Finding Quick Fixes..."
                                                                </div>
                                                            }.into_any()
                                                        } else if !suggs.is_empty() {
                                                            view! {
                                                                <div class="error-floating-suggestions">
                                                                    {suggs.clone().into_iter().map(|sugg| {
                                                                        let title = sugg.title.clone();
                                                                        let replacement = sugg.replacement.clone();
                                                                        let range = sugg.range.clone();
                                                                        let snack_cb = snack;
                                                                        let code_cb = code_sig;
                                                                        let active_error_cb = active_error_sig;
                                                                        
                                                                        let has_fix = replacement.is_some() && range.is_some();
                                                                        
                                                                        let on_apply = move |_| {
                                                                            if let (Some(repl), Some(r)) = (&replacement, &range) {
                                                                                let orig = code_cb.get_untracked();
                                                                                let updated = apply_replacement(&orig, r, repl);
                                                                                code_cb.set(updated);
                                                                                snack_cb.run("Quick Fix applied successfully!".to_string());
                                                                                active_error_cb.set(None);
                                                                            }
                                                                        };
                                                                        
                                                                        view! {
                                                                            <div class="error-floating-suggestion-item">
                                                                                <span class="lightbulb-icon">"💡"</span>
                                                                                <span class="suggestion-text">{title}</span>
                                                                                {has_fix.then(|| view! {
                                                                                    <button class="btn btn-primary btn-xs" on:click=on_apply style="margin-left:auto;padding:2px 6px;font-size:10px">
                                                                                        "Fix"
                                                                                    </button>
                                                                                })}
                                                                            </div>
                                                                        }
                                                                    }).collect_view()}
                                                                </div>
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                <div class="error-floating-no-fix">
                                                                    "No quick fixes available."
                                                                </div>
                                                            }.into_any()
                                                        }
                                                    }}
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { "" }.into_any()
                                        }
                                    }}
                                    <div id="cursor-mirror" style=move || format!(
                                        "width:100%;font-size:{}px;line-height:1.6;tab-size:{}",
                                        settings.get().font_size,
                                        settings.get().tab_size
                                    ) />
                                </div>

                            }
                        }}
                    </div>

                    <BottomPanel 
                        bottom_tab=bottom_tab
                        output=output.into()
                        is_error=is_error.into()
                        show_snack=show_snack
                        diagnostics_list=diagnostics_list.into()
                        on_click_problem=on_click_problem
                        code=code
                        language=Signal::derive(move || project_lang_str.get_value())
                    />

                    <div class="editor-footer">
                        {["TAB","{}","[]","()","\"\"","''","->","=>","::","/ /","/* */"].iter().map(|s| {
                            let s_val = s.replace(" ", "");
                            let s_val_2 = s_val.clone();
                            view! {
                                <button class="btn btn-footer" on:click=move |_| {
                                    let ins = if s_val_2 == "TAB" { " ".repeat(settings.get_untracked().tab_size) } else { s_val_2.clone() };
                                    use wasm_bindgen::JsCast;
                                    if let Some(target) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.query_selector(".code-editor").ok().flatten()) {
                                        if let Ok(target) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                                            let start = target.selection_start().unwrap().unwrap_or(0);
                                            let end = target.selection_end().unwrap().unwrap_or(0);
                                            let val = js_sys::JsString::from(target.value());
                                            code.set(format!("{}{}{}", String::from(val.substring(0, start)), ins, String::from(val.substring(end, val.length()))));
                                            dirty.set(true);
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
                </div>

                {move || preview_url.get().map(|url| {
                    let final_url = move || {
                        let k = refresh_key.get();
                        if url.contains('?') {
                            format!("{}&refresh={}", url, k)
                        } else {
                            format!("{}?refresh={}", url, k)
                        }
                    };
                    view! {
                        <>
                        <div class="preview-resize-gutter"></div>
                        <div class="desktop-preview-panel">
                            <div class="preview-header">
                                <span style="display:inline-flex; align-items:center; gap:6px; color:#fff; font-family:var(--font-ui); font-size:12px; font-weight:600;">
                                    <LucideIcon name="globe" size="14" />
                                    "Live Web Preview"
                                </span>
                                <button class="btn btn-xs" style="padding:2px 8px; font-size:11px; height:24px; background:var(--bg3); border:1px solid var(--border); border-radius:var(--radius-sm); color:var(--text); cursor:pointer; display:inline-flex; align-items:center; gap:4px;"
                                    on:click=move |_| refresh_key.update(|k| *k += 1)>
                                    <LucideIcon name="refresh" size="12" /> "Refresh"
                                </button>
                            </div>
                            <iframe class="preview-frame" src=final_url style="flex:1; border:none; background:#fff; width:100%; height:100%;" />
                        </div>
                        </>
                    }
                })}
            </div>

            <DependencyModal 
                show_deps=show_deps
                dep_input=dep_input
                dep_output=dep_output.into()
                add_dep=add_dep
            />

            {move || show_mobile_full_preview.get().then(|| {
                let url_opt = preview_url.get();
                url_opt.map(|url| {
                    let final_url = move || {
                        let k = refresh_key.get();
                        if url.contains('?') {
                            format!("{}&refresh={}", url, k)
                        } else {
                            format!("{}?refresh={}", url, k)
                        }
                    };
                    view! {
                        <div class="mobile-preview-overlay active">
                            <div class="preview-header">
                                <button class="btn btn-icon" on:click=move |_| show_mobile_full_preview.set(false) title="Back to Code" style="background:transparent; border:none; color:var(--text); cursor:pointer; display:inline-flex; align-items:center; justify-content:center;">
                                    <LucideIcon name="arrow-left" size="20" />
                                </button>
                                <span style="font-weight: 600; color: #fff; font-family: var(--font-ui); font-size: 14px;">"Web Preview"</span>
                                <button class="btn btn-xs" style="padding:4px 10px; font-size:11px; height:26px; background:var(--bg3); border:1px solid var(--border); border-radius:var(--radius-sm); color:var(--text); cursor:pointer; display:inline-flex; align-items:center; gap:4px;"
                                    on:click=move |_| refresh_key.update(|k| *k += 1)>
                                    <LucideIcon name="refresh" size="12" /> "Refresh"
                                </button>
                            </div>
                            <iframe class="preview-frame" src=final_url style="flex:1; border:none; background:#fff; width:100%; height:100%;" />
                        </div>
                    }
                })
            })}

            <Snackbar message=snack_msg.read_only() />
        </div>
    }.into_any()
}

fn resolve_completion(item: &api::CompletionItem) -> (String, Option<usize>) {
    if let Some(ref raw_snippet) = item.insert_text {
        let mut result = String::new();
        let mut cursor_offset = None;
        let chars: Vec<char> = raw_snippet.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            if chars[i] == '$' && i + 1 < chars.len() {
                let next = chars[i + 1];
                if next.is_ascii_digit() {
                    let is_primary = next == '0' || next == '1';
                    if is_primary && cursor_offset.is_none() {
                        cursor_offset = Some(result.encode_utf16().count());
                    }
                    i += 2;
                } else if next == '{' {
                    let mut j = i + 2;
                    let mut content = String::new();
                    while j < chars.len() && chars[j] != '}' {
                        content.push(chars[j]);
                        j += 1;
                    }
                    if j < chars.len() {
                        let placeholder = if let Some(colon_pos) = content.find(':') {
                            &content[colon_pos + 1..]
                        } else {
                            ""
                        };
                        if cursor_offset.is_none() {
                            cursor_offset = Some(result.encode_utf16().count());
                        }
                        result.push_str(placeholder);
                        i = j + 1;
                    } else {
                        result.push('$');
                        i += 1;
                    }
                } else {
                    result.push('$');
                    i += 1;
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        (result, cursor_offset)
    } else {
        let label = &item.label;
        if let Some(pos) = label.find("(...)") {
            let cleaned = label.replace("(...)", "()");
            (cleaned, Some(pos + 1))
        } else if let Some(pos) = label.find("{...}") {
            let cleaned = label.replace("{...}", "{}");
            (cleaned, Some(pos + 1))
        } else if let Some(pos) = label.find("[...]") {
            let cleaned = label.replace("[...]", "[]");
            (cleaned, Some(pos + 1))
        } else {
            (label.clone(), None)
        }
    }
}
