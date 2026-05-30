use leptos::prelude::*;
use crate::api;
use crate::components::icon::LucideIcon;
use pulldown_cmark::{Parser, Options, html};
use super::utils::highlight_code;

pub fn markdown_to_html(markdown: &str) -> String {
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

pub fn build_hover_html(diagnostics: &[api::Diagnostic], hover_markdown: Option<&str>) -> String {
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
pub fn HoverCard(
    hover_visible: RwSignal<bool>,
    hover_coords: RwSignal<(f64, f64)>,
    hover_loading: RwSignal<bool>,
    hover_error: RwSignal<Option<String>>,
    hover_content: RwSignal<Option<String>>,
    hover_card_active: RwSignal<bool>,
    close_hover: Callback<()>,
    trigger_definition: Callback<()>,
    trigger_references: Callback<()>,
) -> impl IntoView {
    view! {
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
                            close_hover.run(());
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
                            <LucideIcon name="locate-fixed" size="14" class="btn-icon" />
                            "Go to Definition"
                        </button>
                        <button class="hover-action-btn" on:click=on_refs_click>
                            <LucideIcon name="search-code" size="14" class="btn-icon" />
                            "Find References"
                        </button>
                    </div>
                </div>
            }
        })}
    }
}
