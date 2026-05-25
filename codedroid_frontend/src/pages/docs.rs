use leptos::prelude::*;
use web_sys::{MouseEvent, window};
use wasm_bindgen::JsCast;
use crate::components::app_bar::AppBar;
use crate::components::icon::LucideIcon;

fn resolve_relative_path(current: &str, relative: &str) -> String {
    if relative.starts_with('/') || relative.contains("://") {
        return relative.to_string();
    }
    
    let mut parts: Vec<&str> = current.split('/').collect();
    if !parts.is_empty() {
        parts.pop();
    }
    
    for segment in relative.split('/') {
        if segment == "." {
            continue;
        } else if segment == ".." {
            if !parts.is_empty() {
                parts.pop();
            }
        } else {
            parts.push(segment);
        }
    }
    
    parts.join("/")
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub fn markdown_to_html(md: &str) -> String {
    use pulldown_cmark::{Parser, Options, Event, Tag, TagEnd, CodeBlockKind};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(md, options);
    let mut events = Vec::new();
    let mut in_code_block = false;
    let mut current_lang = String::new();
    let mut current_code = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                in_code_block = true;
                current_lang = lang.to_string();
                current_code.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                if in_code_block {
                    in_code_block = false;
                    
                    let escaped_code = escape_html(&current_code);
                    let raw_code_attr = current_code.replace('"', "&quot;");
                    
                    let html = format!(
                        r##"<div class="md-code-block">
                            <div class="md-code-header">
                                <span class="md-code-lang">{}</span>
                                <button class="btn-copy-code" data-code="{}">
                                    <svg class="copy-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2.5" fill="none" stroke-linecap="round" stroke-linejoin="round" style="vertical-align: middle; margin-right: 4px;"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                                    <span>Copy</span>
                                </button>
                            </div>
                            <pre><code class="language-{}">{}</code></pre>
                        </div>"##,
                        if current_lang.is_empty() { "code" } else { &current_lang },
                        raw_code_attr,
                        current_lang,
                        escaped_code
                    );
                    events.push(Event::Html(html.into()));
                }
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                let is_md_link = dest_url.ends_with(".md") || dest_url.contains(".md#") || (!dest_url.contains("://") && !dest_url.starts_with('#'));
                if is_md_link {
                    events.push(Event::Html(format!(r##"<a class="md-link" href="#" data-path="{}">"##, dest_url).into()));
                } else {
                    events.push(Event::Html(format!(r##"<a href="{}" target="_blank" rel="noopener noreferrer">"##, dest_url).into()));
                }
            }
            Event::End(TagEnd::Link) => {
                events.push(Event::Html("</a>".into()));
            }
            Event::Text(text) => {
                if in_code_block {
                    current_code.push_str(&text);
                } else {
                    events.push(Event::Text(text));
                }
            }
            other => {
                if in_code_block {
                    // collect code contents
                } else {
                    events.push(other);
                }
            }
        }
    }

    let mut html_buf = String::new();
    pulldown_cmark::html::push_html(&mut html_buf, events.into_iter());
    html_buf
}

#[component]
pub fn DocsPage() -> impl IntoView {
    let docs_list = RwSignal::new(Vec::<String>::new());
    let current_path = RwSignal::new("README.md".to_string());
    let markdown_content = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let error = RwSignal::new(String::new());

    // Fetch documentation file list on mount
    let load_list = move || {
        leptos::task::spawn_local(async move {
            match crate::api::list_docs_api().await {
                Ok(res) => {
                    if res.error.is_empty() {
                        docs_list.set(res.files);
                    }
                }
                Err(_) => {}
            }
        });
    };

    // Load selected doc
    let load_doc = move |path: String| {
        loading.set(true);
        error.set(String::new());
        let path_clone = path.clone();
        
        leptos::task::spawn_local(async move {
            match crate::api::read_doc_api(&path_clone).await {
                Ok(res) => {
                    if !res.error.is_empty() {
                        error.set(res.error);
                    } else {
                        markdown_content.set(res.content);
                    }
                }
                Err(e) => {
                    error.set(e);
                }
            }
            loading.set(false);
        });
    };

    // Fetch file list once on load
    Effect::new(move |_| {
        load_list();
    });

    // React to path changes
    Effect::new(move |_| {
        let path = current_path.get();
        load_doc(path);
    });

    // Handle dynamic link and copy button clicks
    let handle_click = move |e: MouseEvent| {
        let Some(target) = e.target() else { return; };
        let Ok(element) = target.dyn_into::<web_sys::Element>() else { return; };
        
        let mut current: Option<web_sys::Element> = Some(element.clone());
        while let Some(el) = current {
            if el.class_list().contains("btn-copy-code") {
                if let Some(code) = el.get_attribute("data-code") {
                    if let Some(win) = window() {
                        let _ = win.navigator().clipboard().write_text(&code);
                        if let Some(span) = el.query_selector("span").ok().flatten() {
                            let span_el = span.dyn_into::<web_sys::HtmlElement>().unwrap();
                            let old_text = span_el.text_content().unwrap_or_else(|| "Copy".to_string());
                            span_el.set_text_content(Some("Copied!"));
                            let span_clone = span_el.clone();
                            gloo_timers::callback::Timeout::new(1500, move || {
                                span_clone.set_text_content(Some(&old_text));
                            }).forget();
                        }
                    }
                }
                e.prevent_default();
                return;
            }
            
            if el.class_list().contains("md-link") {
                if let Some(rel_path) = el.get_attribute("data-path") {
                    let resolved = resolve_relative_path(&current_path.get_untracked(), &rel_path);
                    current_path.set(resolved);
                }
                e.prevent_default();
                return;
            }
            
            current = el.parent_element();
        }
    };

    view! {
        <div class="docs-page-root">
            <AppBar title="Documentation".to_string() back=true />

            <div class="docs-layout">
                // Sidebar
                <div class="docs-sidebar">
                    <div class="docs-sidebar-header">
                        <LucideIcon name="book" size="18" />
                        <span>"Workspace Documents"</span>
                    </div>
                    <div class="docs-sidebar-list">
                        {move || {
                            docs_list.get().into_iter().map(|file| {
                                let file_clone = file.clone();
                                let is_active = move || current_path.get() == file_clone;
                                let f_c = file.clone();
                                view! {
                                    <div
                                        class=move || if is_active() { "docs-sidebar-item active" } else { "docs-sidebar-item" }
                                        on:click=move |_| current_path.set(f_c.clone())
                                    >
                                        <LucideIcon name="file" size="14" />
                                        <span>{file.clone()}</span>
                                    </div>
                                }
                            }).collect_view()
                        }}
                    </div>
                </div>

                // Content Panel
                <div class="docs-content-container">
                    {move || {
                        if loading.get() {
                            view! {
                                <div class="docs-status-container">
                                    <div class="spinner"></div>
                                    <p>"Loading documentation..."</p>
                                </div>
                            }.into_any()
                        } else if !error.get().is_empty() {
                            view! {
                                <div class="docs-status-container">
                                    <div style="color:var(--danger);font-size:32px;margin-bottom:12px">
                                        <LucideIcon name="alert-triangle" size="48" />
                                    </div>
                                    <p style="color:var(--danger)">{error.get()}</p>
                                </div>
                            }.into_any()
                        } else {
                            let raw_html = markdown_to_html(&markdown_content.get());
                            view! {
                                <div class="docs-markdown-body"
                                    on:click=handle_click
                                    inner_html=raw_html
                                />
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
