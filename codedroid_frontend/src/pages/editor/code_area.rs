use leptos::prelude::*;
use crate::models::Settings;
use crate::pages::editor::utils::*;
use crate::pages::editor::components::apply_replacement;
use crate::api;
use wasm_bindgen_futures::spawn_local;

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
                            on:keydown=move |e: web_sys::KeyboardEvent| {
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
                                <div 
                                    class="suggestions-floating" 
                                    on:mousedown=move |e: web_sys::MouseEvent| { e.prevent_default(); }
                                    style=format!("left:{}px; top:{}px", coords.0, coords.1)
                                >
                                    {move || suggestions.get().into_iter().enumerate().map(|(i, s)| {
                                        let s2 = s.clone();
                                        let s3 = s.clone();
                                        view! {
                                            <button 
                                                class=move || if selected_idx.get() == i { "suggestion-item selected" } else { "suggestion-item" }
                                                on:mousedown=move |e: web_sys::MouseEvent| { e.prevent_default(); }
                                                on:mouseup=move |e: web_sys::MouseEvent| { e.prevent_default(); on_select.run(s2.clone()); }
                                                on:click=move |e: web_sys::MouseEvent| { e.prevent_default(); }
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
    }
}
