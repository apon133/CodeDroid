use super::utils::highlight_code;
use crate::api;
use leptos::prelude::*;
use pulldown_cmark::{html, Options, Parser};

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
                    let html_block =
                        format!("<div class=\"hover-code-block\">{}</div>", highlighted);
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

pub fn build_hover_html(_diagnostics: &[api::Diagnostic], hover_markdown: Option<&str>) -> String {
    let mut html = String::new();

    // Render hover markdown only (removing diagnostic error details from documentation tooltip)
    if let Some(md) = hover_markdown {
        let md_html = markdown_to_html(md);
        html.push_str(&format!(
            "<div class=\"hover-markdown-content\">{}</div>",
            md_html
        ));
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

            let _ = trigger_definition;
            let _ = trigger_references;

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
                </div>
            }
        })}
    }
}
