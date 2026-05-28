use leptos::prelude::*;
use crate::models::Settings;
use crate::pages::editor::utils::*;
use crate::pages::editor::components::apply_replacement;
use crate::api;
use wasm_bindgen_futures::spawn_local;
use pulldown_cmark::{Parser, Options, html};

fn markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{Event, Tag};
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    
    let parser = Parser::new_ext(markdown, options);
    let mut new_events = Vec::new();
    
    let mut in_code_block = false;
    let mut code_block_lang = String::new();
    let mut code_block_content = String::new();
    
    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(ref kind)) => {
                in_code_block = true;
                code_block_lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                    pulldown_cmark::CodeBlockKind::Indented => String::new(),
                };
                code_block_content.clear();
            }
            Event::End(end_tag) => {
                if in_code_block {
                    in_code_block = false;
                    let highlighted = highlight_code(&code_block_content, &code_block_lang);
                    let html_block = format!("<div class=\"hover-code-block\">{}</div>", highlighted);
                    new_events.push(Event::Html(html_block.into()));
                } else {
                    new_events.push(Event::End(end_tag));
                }
            }
            Event::Text(text) => {
                if in_code_block {
                    code_block_content.push_str(&text);
                } else {
                    new_events.push(Event::Text(text));
                }
            }
            other => {
                if in_code_block {
                    match &other {
                        Event::SoftBreak | Event::HardBreak => {
                            code_block_content.push('\n');
                        }
                        _ => {}
                    }
                } else {
                    new_events.push(other);
                }
            }
        }
    }
    
    let mut html_output = String::new();
    html::push_html(&mut html_output, new_events.into_iter());
    html_output
}

fn build_hover_html(diagnostics: &[api::Diagnostic], hover_markdown: Option<&str>) -> String {
    let mut html = String::new();
    
    // 1. Render diagnostics
    if !diagnostics.is_empty() {
        html.push_str("<div class=\"hover-diagnostics-container\">");
        for diag in diagnostics {
            let severity_val = diag.severity.unwrap_or(1);
            let (sev_class, sev_label) = match severity_val {
                1 => ("error", "Error"),
                2 => ("warning", "Warning"),
                3 => ("info", "Info"),
                4 => ("hint", "Hint"),
                _ => ("error", "Error"),
            };
            
            let source_str = diag.source.as_deref().unwrap_or("LSP");
            let code_str = diag.code.as_ref()
                .and_then(|c| {
                    if c.is_string() {
                        c.as_str().map(|s| s.to_string())
                    } else if c.is_number() {
                        c.as_i64().map(|n| n.to_string())
                    } else {
                        None
                    }
                });
                
            let source_html = if let Some(code) = code_str {
                format!("<span class=\"hover-diagnostic-source\">{}[{}]</span>", source_str, code)
            } else {
                format!("<span class=\"hover-diagnostic-source\">{}</span>", source_str)
            };
            
            html.push_str(&format!(
                "<div class=\"hover-diagnostic-item diag-{}\">\
                    <div class=\"hover-diagnostic-header\">\
                        <span class=\"hover-diagnostic-badge\">{}</span>\
                        {}\
                    </div>\
                    <div class=\"hover-diagnostic-message\">{}</div>\
                </div>",
                sev_class,
                sev_label,
                source_html,
                diag.message.replace('<', "&lt;").replace('>', "&gt;")
            ));
        }
        html.push_str("</div>");
    }
    
    // 2. Render divider if we have both
    if !diagnostics.is_empty() && hover_markdown.is_some() {
        html.push_str("<div class=\"hover-divider\"></div>");
    }
    
    // 3. Render hover markdown
    if let Some(md) = hover_markdown {
        let md_html = markdown_to_html(md);
        html.push_str(&format!("<div class=\"hover-markdown-content\">{}</div>", md_html));
    }
    
    html
}

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
                                    let matching_diags: Vec<api::Diagnostic> = diags.into_iter()
                                        .filter(|d| {
                                            let file_matches = d.file.is_none() || d.file.as_ref() == active_file.as_ref();
                                            if !file_matches { return false; }
                                            
                                            if l >= d.range.start.line && l <= d.range.end.line {
                                                if l == d.range.start.line && l == d.range.end.line {
                                                    c >= d.range.start.character && c <= d.range.end.character
                                                } else if l == d.range.start.line {
                                                    c >= d.range.start.character
                                                } else if l == d.range.end.line {
                                                    c <= d.range.end.character
                                                } else {
                                                    true
                                                }
                                            } else {
                                                false
                                            }
                                        })
                                        .collect();
                                    
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
                            let matching_diags: Vec<api::Diagnostic> = diags.into_iter()
                                .filter(|d| {
                                    let file_matches = d.file.is_none() || d.file.as_ref() == active_file.as_ref();
                                    if !file_matches { return false; }
                                    
                                    if line >= d.range.start.line && line <= d.range.end.line {
                                        if line == d.range.start.line && line == d.range.end.line {
                                            character >= d.range.start.character && character <= d.range.end.character
                                        } else if line == d.range.start.line {
                                            character >= d.range.start.character
                                        } else if line == d.range.end.line {
                                            character <= d.range.end.character
                                        } else {
                                            true
                                        }
                                    } else {
                                        false
                                    }
                                })
                                .collect();
                            
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
                        {move || hover_visible.get().then(|| {
                            let coords = hover_coords.get();
                            
                            let is_mobile = web_sys::window()
                                .and_then(|w| w.inner_width().ok())
                                .and_then(|w| w.as_f64())
                                .map(|w| w < 768.0)
                                .unwrap_or(false);
                                
                            let hover_class = if is_mobile {
                                "hover-bottom-sheet"
                            } else {
                                "hover-floating"
                            };
                            
                            let style = if is_mobile {
                                "".to_string()
                            } else {
                                format!("left:{}px; top:{}px", coords.0, coords.1)
                            };
                            
                            let close_click = move |e: web_sys::MouseEvent| {
                                e.prevent_default();
                                e.stop_propagation();
                                hover_visible.set(false);
                            };
                            
                            let trigger_definition_c = trigger_definition.clone();
                            let on_def_click = move |e: web_sys::MouseEvent| {
                                e.prevent_default();
                                e.stop_propagation();
                                hover_visible.set(false);
                                trigger_definition_c.run(());
                            };
                            
                            let trigger_references_c = trigger_references.clone();
                            let on_refs_click = move |e: web_sys::MouseEvent| {
                                e.prevent_default();
                                e.stop_propagation();
                                hover_visible.set(false);
                                trigger_references_c.run(());
                            };
                            
                            view! {
                                <div
                                    class=hover_class
                                    style=style
                                    on:mouseenter={
                                        let hover_card_active = hover_card_active.clone();
                                        move |_| {
                                            hover_card_active.set(true);
                                        }
                                    }
                                    on:mouseleave={
                                        let hover_card_active = hover_card_active.clone();
                                        let close_hover = close_hover.clone();
                                        move |_| {
                                            hover_card_active.set(false);
                                            close_hover();
                                        }
                                    }
                                >
                                    <div class="hover-header">
                                        <span class="hover-header-title">"Documentation"</span>
                                        <button class="hover-close-btn" on:click=close_click>
                                            "✕"
                                        </button>
                                    </div>
                                    <div class="hover-content">
                                        {move || {
                                            if hover_loading.get() {
                                                view! {
                                                    <div class="hover-loading-wrap">
                                                        <div class="spinner" style="width:12px;height:12px;border-width:1.5px" />
                                                        <span>"Loading docs..."</span>
                                                    </div>
                                                }.into_any()
                                            } else if let Some(err) = hover_error.get() {
                                                view! {
                                                    <div class="hover-error-wrap">
                                                        {format!("Error: {}", err)}
                                                    </div>
                                                }.into_any()
                                            } else if let Some(html_content) = hover_content.get() {
                                                view! {
                                                    <div inner_html=html_content />
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div style="color:var(--text2)">"No documentation available."</div>
                                                }.into_any()
                                            }
                                        }}
                                    </div>
                                    <div class="hover-footer">
                                        <button class="hover-action-btn" on:click=on_def_click>
                                            <span class="btn-icon">"🔍"</span>
                                            "Go to Definition"
                                        </button>
                                        <button class="hover-action-btn" on:click=on_refs_click>
                                            <span class="btn-icon">"📚"</span>
                                            "Find References"
                                        </button>
                                    </div>
                                </div>
                            }
                        })}
                    </div>
                }
            }}
        </div>
    }
}
