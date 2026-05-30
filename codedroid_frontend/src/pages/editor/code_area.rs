use leptos::prelude::*;
use crate::models::Settings;
use super::utils::*;
use crate::api;
use wasm_bindgen_futures::spawn_local;

use super::hover::{HoverCard, build_hover_html};
use super::suggestions::SuggestionsOverlay;
use super::error_popover::ErrorPopover;

#[component]
pub fn EditorCodeArea(
    settings: RwSignal<Settings>,
    code: RwSignal<String>,
    dirty: RwSignal<bool>,
    active_tab: RwSignal<Option<String>>,
    diagnostics_list: RwSignal<Vec<api::Diagnostic>>,
    active_error: RwSignal<Option<(api::Diagnostic, Vec<api::CodeSuggestion>, bool)>>,
    cursor_pos: RwSignal<u32>,
    cursor_coords: RwSignal<(f64, f64)>,
    suggestions: RwSignal<Vec<api::CompletionItem>>,
    selected_idx: RwSignal<usize>,
    project_lang_str: StoredValue<String>,
    project_path_str: StoredValue<String>,
    last_request_id: RwSignal<u64>,
    trigger_diagnostics: Callback<String>,
    save_current: Callback<()>,
    format_code: Callback<()>,
    show_search: RwSignal<bool>,
    check_error_at_cursor: Callback<(u32, u32)>,
    on_select: Callback<api::CompletionItem>,
    show_snack: Callback<String>,
    trigger_definition: Callback<()>,
    trigger_references: Callback<()>,
) -> impl IntoView {
    let hover_visible = RwSignal::new(false);
    let hover_content = RwSignal::new(None::<String>);
    let hover_coords = RwSignal::new((0.0, 0.0));
    let hover_loading = RwSignal::new(false);
    let hover_error = RwSignal::new(None::<String>);
    
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();
    let mouse_coords = RwSignal::new((0.0, 0.0));
    let hover_version = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let hover_card_active = RwSignal::new(false);

    view! {
        <div class="code-area" style="flex:2">
            {move || {
                let s = settings.get();
                let content = code.get();
                let ext = active_tab.get().map(|n| file_extension(&n).to_string()).unwrap_or_default();
                let highlighted_lines = highlight_code_lines(&content, &ext);
                let active = active_tab.get();
                let diags = diagnostics_list.get();
                let active_diags: Vec<api::Diagnostic> = diags.into_iter()
                    .filter(|d| d.file.is_none() || d.file.as_ref() == active.as_ref())
                    .collect();

                let close_hover_immediate = {
                    let hover_version = hover_version.clone();
                    let hover_visible = hover_visible.clone();
                    move || {
                        hover_version.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        hover_visible.set(false);
                    }
                };

                let close_hover = {
                    let hover_version = hover_version.clone();
                    let hover_visible = hover_visible.clone();
                    let hover_card_active = hover_card_active.clone();
                    move || {
                        hover_version.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let hover_visible_c = hover_visible.clone();
                        let hover_card_active_c = hover_card_active.clone();
                        spawn_local(async move {
                            gloo_timers::future::TimeoutFuture::new(150).await;
                            if !hover_card_active_c.get() {
                                hover_visible_c.set(false);
                            }
                        });
                    }
                };

                let trigger_hover = {
                    let active_tab = active_tab.clone();
                    let settings = settings.clone();
                    let project_path_str = project_path_str.clone();
                    let suggestions = suggestions.clone();
                    let textarea_ref = textarea_ref.clone();
                    let hover_loading = hover_loading.clone();
                    let hover_visible = hover_visible.clone();
                    let hover_error = hover_error.clone();
                    let hover_content = hover_content.clone();
                    let hover_coords = hover_coords.clone();
                    let hover_card_active = hover_card_active.clone();
                    let diagnostics_list = diagnostics_list.clone();
                    move |mouse_x: f64, mouse_y: f64| {
                        if !suggestions.get_untracked().is_empty() || hover_card_active.get_untracked() {
                            return;
                        }
                        if let Some(target) = textarea_ref.get() {
                            use wasm_bindgen::JsCast;
                            let target_el: &web_sys::HtmlTextAreaElement = &target;
                            let rect = target_el.unchecked_ref::<web_sys::Element>().get_bounding_client_rect();
                            let val = target_el.value();
                            let font_size = settings.get_untracked().font_size as f64;
                            
                            let line_height = font_size * 1.6;
                            let char_width = font_size * 0.602;
                            
                            let style = web_sys::window().unwrap().get_computed_style(target_el).unwrap().unwrap();
                            let padding_left = style.get_property_value("padding-left").unwrap_or_default()
                                .replace("px", "").parse::<f64>().unwrap_or(0.0);
                            let padding_top = style.get_property_value("padding-top").unwrap_or_default()
                                .replace("px", "").parse::<f64>().unwrap_or(0.0);
                            
                            let relative_x = mouse_x - rect.left() - padding_left + target_el.scroll_left() as f64;
                            let relative_y = mouse_y - rect.top() - padding_top + target_el.scroll_top() as f64;
                            
                            let line_idx = (relative_y / line_height).floor() as i32;
                            let char_idx = (relative_x / char_width).round() as i32;
                            
                            if line_idx >= 0 && char_idx >= 0 {
                                let lines: Vec<&str> = val.split('\n').collect();
                                if (line_idx as usize) < lines.len() {
                                    let line_text = lines[line_idx as usize];
                                    let char_idx = char_idx.min(line_text.chars().count() as i32);
                                    
                                    let char_at = line_text.chars().nth(char_idx as usize);
                                    let is_word_char = char_at.map(|c: char| c.is_alphanumeric() || c == '_' || c == '.' || c == ':').unwrap_or(false);
                                    
                                    let l = line_idx as u32;
                                    let c = char_idx as u32;
                                    
                                    let active_file = active_tab.get_untracked();
                                    let diags = diagnostics_list.get_untracked();
                                    let matching_diags = get_matching_diagnostics(&diags, active_file.as_ref(), l, c);
                                    
                                    let has_diags = !matching_diags.is_empty();
                                    
                                    if is_word_char || has_diags {
                                        hover_loading.set(is_word_char);
                                        hover_visible.set(true);
                                        hover_error.set(None);
                                        
                                        let card_x = mouse_x - rect.left();
                                        let card_y = mouse_y - rect.top() + 20.0;
                                        hover_coords.set((card_x, card_y));
                                        
                                        if is_word_char {
                                            let active_file = active_tab.get_untracked().unwrap_or_default();
                                            let lang = file_to_lsp_lang(&active_file);
                                            let proj_path = project_path_str.get_value();
                                            let code_content = val.clone();
                                            let matching_diags_clone = matching_diags.clone();
                                            let hover_content = hover_content.clone();
                                            let hover_loading = hover_loading.clone();
                                            let hover_visible = hover_visible.clone();
                                            let hover_error = hover_error.clone();
                                            
                                            if !matching_diags_clone.is_empty() {
                                                let initial_html = build_hover_html(&matching_diags_clone, None);
                                                hover_content.set(Some(initial_html));
                                            } else {
                                                hover_content.set(None);
                                            }
                                            
                                            spawn_local(async move {
                                                match api::hover_api(&proj_path, &active_file, &code_content, l, c, &lang).await {
                                                    Ok(res) => {
                                                        hover_loading.set(false);
                                                        if let Some(contents) = res.contents {
                                                            if !contents.trim().is_empty() {
                                                                let full_html = build_hover_html(&matching_diags_clone, Some(&contents));
                                                                hover_content.set(Some(full_html));
                                                            } else if !matching_diags_clone.is_empty() {
                                                                let final_html = build_hover_html(&matching_diags_clone, None);
                                                                hover_content.set(Some(final_html));
                                                            } else {
                                                                hover_visible.set(false);
                                                            }
                                                        } else if !matching_diags_clone.is_empty() {
                                                            let final_html = build_hover_html(&matching_diags_clone, None);
                                                            hover_content.set(Some(final_html));
                                                        } else {
                                                            hover_visible.set(false);
                                                        }
                                                    }
                                                    Err(e) => {
                                                        hover_loading.set(false);
                                                        if !matching_diags_clone.is_empty() {
                                                            let final_html = build_hover_html(&matching_diags_clone, None);
                                                            hover_content.set(Some(final_html));
                                                        } else {
                                                            hover_error.set(Some(e));
                                                        }
                                                    }
                                                }
                                            });
                                        } else {
                                            let html = build_hover_html(&matching_diags, None);
                                            hover_content.set(Some(html));
                                        }
                                    } else {
                                        hover_visible.set(false);
                                    }
                                } else {
                                    hover_visible.set(false);
                                }
                            } else {
                                hover_visible.set(false);
                            }
                        }
                    }
                };

                let trigger_hover_at_cursor = {
                    let active_tab = active_tab.clone();
                    let project_path_str = project_path_str.clone();
                    let cursor_coords = cursor_coords.clone();
                    let suggestions = suggestions.clone();
                    let textarea_ref = textarea_ref.clone();
                    let hover_loading = hover_loading.clone();
                    let hover_visible = hover_visible.clone();
                    let hover_error = hover_error.clone();
                    let hover_content = hover_content.clone();
                    let hover_coords = hover_coords.clone();
                    let diagnostics_list = diagnostics_list.clone();
                    move |cursor_idx: u32| {
                        if !suggestions.get_untracked().is_empty() {
                            return;
                        }
                        if let Some(target) = textarea_ref.get() {
                            let target_el: &web_sys::HtmlTextAreaElement = &target;
                            let val = target_el.value();
                            
                            let (line, character) = {
                                let before_cursor = val.chars().take(cursor_idx as usize).collect::<String>();
                                let lines: Vec<&str> = before_cursor.split('\n').collect();
                                (lines.len().saturating_sub(1) as u32, lines.last().map(|l: &&str| (*l).chars().count()).unwrap_or(0) as u32)
                            };
                            
                            let char_at = val.chars().nth(cursor_idx as usize);
                            let is_word_char = char_at.map(|c: char| c.is_alphanumeric() || c == '_' || c == '.' || c == ':').unwrap_or(false);
                            
                            let active_file = active_tab.get_untracked();
                            let diags = diagnostics_list.get_untracked();
                            let matching_diags = get_matching_diagnostics(&diags, active_file.as_ref(), line, character);
                            
                            let has_diags = !matching_diags.is_empty();
                            
                            if is_word_char || has_diags {
                                hover_loading.set(is_word_char);
                                hover_visible.set(true);
                                hover_error.set(None);
                                
                                let coords = cursor_coords.get_untracked();
                                hover_coords.set((coords.0, coords.1 + 10.0));
                                
                                if is_word_char {
                                    let active_file = active_tab.get_untracked().unwrap_or_default();
                                    let lang = file_to_lsp_lang(&active_file);
                                    let proj_path = project_path_str.get_value();
                                    let code_content = val.clone();
                                    let matching_diags_clone = matching_diags.clone();
                                    let hover_content = hover_content.clone();
                                    let hover_loading = hover_loading.clone();
                                    let hover_visible = hover_visible.clone();
                                    let hover_error = hover_error.clone();
                                    
                                    if !matching_diags_clone.is_empty() {
                                        let initial_html = build_hover_html(&matching_diags_clone, None);
                                        hover_content.set(Some(initial_html));
                                    } else {
                                        hover_content.set(None);
                                    }
                                    
                                    spawn_local(async move {
                                        match api::hover_api(&proj_path, &active_file, &code_content, line, character, &lang).await {
                                            Ok(res) => {
                                                hover_loading.set(false);
                                                if let Some(contents) = res.contents {
                                                    if !contents.trim().is_empty() {
                                                        let full_html = build_hover_html(&matching_diags_clone, Some(&contents));
                                                        hover_content.set(Some(full_html));
                                                    } else if !matching_diags_clone.is_empty() {
                                                        let final_html = build_hover_html(&matching_diags_clone, None);
                                                        hover_content.set(Some(final_html));
                                                    } else {
                                                        hover_visible.set(false);
                                                    }
                                                } else if !matching_diags_clone.is_empty() {
                                                    let final_html = build_hover_html(&matching_diags_clone, None);
                                                    hover_content.set(Some(final_html));
                                                } else {
                                                    hover_visible.set(false);
                                                }
                                            }
                                            Err(e) => {
                                                hover_loading.set(false);
                                                if !matching_diags_clone.is_empty() {
                                                    let final_html = build_hover_html(&matching_diags_clone, None);
                                                    hover_content.set(Some(final_html));
                                                } else {
                                                    hover_error.set(Some(e));
                                                }
                                            }
                                        }
                                    });
                                } else {
                                    let html = build_hover_html(&matching_diags, None);
                                    hover_content.set(Some(html));
                                }
                            }
                        }
                    }
                };

                let on_mousemove = {
                    let trigger_hover = trigger_hover.clone();
                    let mouse_coords = mouse_coords.clone();
                    let hover_version = hover_version.clone();
                    let suggestions = suggestions.clone();
                    let hover_card_active = hover_card_active.clone();
                    move |e: web_sys::MouseEvent| {
                        if !suggestions.get().is_empty() || hover_card_active.get() {
                            return;
                        }
                        let cx = e.client_x() as f64;
                        let cy = e.client_y() as f64;
                        mouse_coords.set((cx, cy));
                        
                        let current_version = hover_version.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                        let trigger_hover_c = trigger_hover.clone();
                        let hover_version_c = hover_version.clone();
                        spawn_local(async move {
                            gloo_timers::future::TimeoutFuture::new(400).await;
                            if hover_version_c.load(std::sync::atomic::Ordering::SeqCst) == current_version {
                                trigger_hover_c(cx, cy);
                            }
                        });
                    }
                };

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
                                let has_error = active_diags.iter().any(|d| d.range.start.line == (n - 1) as u32 && d.severity.unwrap_or(1) == 1);
                                let has_warning = active_diags.iter().any(|d| d.range.start.line == (n - 1) as u32 && d.severity.unwrap_or(1) == 2);
                                
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
                            node_ref=textarea_ref
                            class="code-layer code-editor"
                            spellcheck="false"
                            prop:value=move || code.get()
                            on:mousemove={
                                let on_mousemove = on_mousemove.clone();
                                move |e| on_mousemove(e)
                            }
                            on:mouseleave={
                                let close_hover = close_hover.clone();
                                move |_| close_hover()
                            }
                            on:beforeinput={
                                let close_hover_immediate = close_hover_immediate.clone();
                                move |input_ev: web_sys::InputEvent| {
                                    close_hover_immediate();
                                    use wasm_bindgen::JsCast;
                                    if let Some(data) = input_ev.data() {
                                        if data.chars().count() == 1 {
                                            let target = input_ev.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                            let start = target.selection_start().unwrap().unwrap_or(0);
                                            let end = target.selection_end().unwrap().unwrap_or(0);
                                            let val = target.value();
                                            if let Some((new_val, new_start, new_end)) = handle_auto_close_pairs(&val, start, end, &data) {
                                                input_ev.prevent_default();
                                                if val != new_val {
                                                    code.set(new_val);
                                                    dirty.set(true);
                                                }
                                                spawn_local(async move {
                                                    let _ = gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                                                    let _ = target.set_selection_range(new_start, new_end);
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                            on:input={
                                 let close_hover_immediate = close_hover_immediate.clone();
                                 move |e: web_sys::Event| {
                                     close_hover_immediate();
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
                                    
                                    if let Some(coords) = update_cursor_coords(&val, start) {
                                        cursor_coords.set(coords);
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
                                            let rel_file = active_file.clone();
                                            spawn_local(async move {
                                                gloo_timers::future::TimeoutFuture::new(150).await;
                                                if last_request_id.get_untracked() == req_id {
                                                    if let Ok(resp) = api::get_completions_api(&val, &lang, &path, &rel_file, line, character).await {
                                                        if last_request_id.get_untracked() == req_id { suggestions.set(resp.suggestions); }
                                                    }
                                                }
                                            });
                                        } else { suggestions.set(Vec::new()); }
                                    } else { suggestions.set(Vec::new()); }
                                }
                            }
                            on:keydown={
                                 let close_hover_immediate = close_hover_immediate.clone();
                                 move |e: web_sys::KeyboardEvent| {
                                     close_hover_immediate();
                                if (e.ctrl_key() || e.meta_key()) && e.key() == "s" { e.prevent_default(); save_current.run(()); }
                                if e.shift_key() && e.alt_key() && (e.key() == "f" || e.key() == "F") { e.prevent_default(); format_code.run(()); }
                                if (e.ctrl_key() || e.meta_key()) && e.key() == "f" { e.prevent_default(); show_search.update(|v| *v = !*v); }
                                if e.key() == "F12" { e.prevent_default(); trigger_definition.run(()); }
                                if e.shift_key() && e.key() == "F12" { e.prevent_default(); trigger_references.run(()); }
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
                                    let rel_file = active_file.clone();
                                    spawn_local(async move {
                                        if let Ok(resp) = api::get_completions_api(&val, &lang, &path, &rel_file, line, character).await {
                                            suggestions.set(resp.suggestions);
                                        }
                                    });
                                }
                                use wasm_bindgen::JsCast;
                                let target = e.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                let start = target.selection_start().unwrap().unwrap_or(0);
                                let end = target.selection_end().unwrap().unwrap_or(0);
                                let val = target.value();

                                if e.key() == "Tab" {
                                    e.prevent_default();
                                    let (new_val, new_pos) = handle_tab_insertion(&val, start, end, settings.get_untracked().tab_size);
                                    code.set(new_val);
                                    dirty.set(true);
                                    spawn_local(async move {
                                        let _ = gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                                        let _ = target.set_selection_range(new_pos, new_pos);
                                    });
                                } else {
                                    let key = e.key();
                                    if let Some((new_val, new_start, new_end)) = handle_auto_close_pairs(&val, start, end, &key) {
                                        e.prevent_default();
                                        if val != new_val {
                                            code.set(new_val);
                                            dirty.set(true);
                                        }
                                        spawn_local(async move {
                                            let _ = gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                                            let _ = target.set_selection_range(new_start, new_end);
                                        });
                                    } else if key == "Backspace" {
                                        if let Some((new_val, new_pos)) = handle_backspace_pairs(&val, start, end) {
                                            e.prevent_default();
                                            code.set(new_val);
                                            dirty.set(true);
                                            spawn_local(async move {
                                                let _ = gloo_timers::future::sleep(std::time::Duration::from_millis(10)).await;
                                                let _ = target.set_selection_range(new_pos, new_pos);
                                            });
                                        }
                                    }
                                }
                            }
                            }
                            on:click={
                                 let close_hover_immediate = close_hover_immediate.clone();
                                 let trigger_hover_at_cursor = trigger_hover_at_cursor.clone();
                                 move |e: web_sys::MouseEvent| {
                                     close_hover_immediate();
                                    suggestions.set(Vec::new());
                                    use wasm_bindgen::JsCast;
                                    let target = e.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                    let start = target.selection_start().unwrap().unwrap_or(0);
                                    cursor_pos.set(start);
                                    let val = target.value();
                                    
                                    if let Some(coords) = update_cursor_coords(&val, start) {
                                        cursor_coords.set(coords);
                                    }

                                    let (line, character) = {
                                        let text_before = &val[..start as usize];
                                        let lines: Vec<&str> = text_before.split('\n').collect();
                                        (lines.len().saturating_sub(1) as u32, lines.last().map(|l| l.chars().count()).unwrap_or(0) as u32)
                                    };
                                    check_error_at_cursor.run((line, character));
                                    trigger_hover_at_cursor(start);
                                }
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
                                    
                                    if let Some(coords) = update_cursor_coords(&val, start) {
                                        cursor_coords.set(coords);
                                    }

                                    let (line, character) = {
                                        let text_before = &val[..start as usize];
                                        let lines: Vec<&str> = text_before.split('\n').collect();
                                        (lines.len().saturating_sub(1) as u32, lines.last().map(|l| l.chars().count()).unwrap_or(0) as u32)
                                    };
                                    check_error_at_cursor.run((line, character));
                                }
                            }
                            on:scroll=move |e: web_sys::Event| {
                                use wasm_bindgen::JsCast;
                                let textarea = e.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>();
                                let scroll_top = textarea.scroll_top();
                                let scroll_left = textarea.scroll_left();
                                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                    if let Some(highlight) = doc.query_selector(".code-highlight").ok().flatten() {
                                        let _ = highlight.unchecked_ref::<web_sys::HtmlElement>().style()
                                            .set_property("transform", &format!("translate({}px, {}px)", -scroll_left, -scroll_top));
                                    }
                                }
                                let start = textarea.selection_start().unwrap().unwrap_or(0);
                                let val = textarea.value();
                                if let Some(coords) = update_cursor_coords(&val, start) {
                                    cursor_coords.set(coords);
                                }
                            }
                        />
                        <SuggestionsOverlay
                            cursor_coords=cursor_coords
                            suggestions=suggestions
                            selected_idx=selected_idx
                            on_select=on_select
                        />
                        <ErrorPopover
                            cursor_coords=cursor_coords
                            active_error=active_error
                            code=code
                            show_snack=show_snack
                        />
                        <div id="cursor-mirror" style=move || format!(
                            "width:100%;font-size:{}px;line-height:1.6;tab-size:{}",
                            settings.get().font_size,
                            settings.get().tab_size
                        ) />
                        <HoverCard
                            hover_visible=hover_visible
                            hover_coords=hover_coords
                            hover_loading=hover_loading
                            hover_error=hover_error
                            hover_content=hover_content
                            hover_card_active=hover_card_active
                            close_hover=Callback::new({
                                let close_hover = close_hover.clone();
                                move |_| close_hover()
                            })
                            trigger_definition=trigger_definition
                            trigger_references=trigger_references
                        />
                    </div>
                }
            }}
        </div>
    }
}
