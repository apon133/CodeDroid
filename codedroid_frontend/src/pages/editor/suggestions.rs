use super::utils::kind_icon;
use crate::api;
use leptos::prelude::*;

#[component]
pub fn SuggestionsOverlay(
    cursor_coords: RwSignal<(f64, f64)>,
    suggestions: RwSignal<Vec<api::CompletionItem>>,
    selected_idx: RwSignal<usize>,
    on_select: Callback<api::CompletionItem>,
) -> impl IntoView {
    let show_docs = RwSignal::new({
        if let Some(window) = web_sys::window() {
            if let Ok(width_val) = window.inner_width() {
                if let Some(w) = width_val.as_f64() {
                    w > 768.0
                } else {
                    true
                }
            } else {
                true
            }
        } else {
            true
        }
    });

    Effect::new(move |_| {
        let _idx = selected_idx.get();
        let _sug_count = suggestions.get().len();
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(container) = document
                    .query_selector(".suggestions-list-container")
                    .ok()
                    .flatten()
                {
                    if let Some(selected_el) = document
                        .query_selector(".suggestion-item.selected")
                        .ok()
                        .flatten()
                    {
                        use wasm_bindgen::JsCast;
                        if let Some(el) = selected_el.dyn_ref::<web_sys::HtmlElement>() {
                            if let Some(container_el) = container.dyn_ref::<web_sys::HtmlElement>()
                            {
                                let item_top = el.offset_top();
                                let item_bottom = item_top + el.offset_height();
                                let scroll_top = container_el.scroll_top();
                                let client_height = container_el.client_height();
                                let scroll_bottom = scroll_top + client_height;

                                if item_top < scroll_top {
                                    container_el.set_scroll_top(item_top);
                                } else if item_bottom > scroll_bottom {
                                    container_el.set_scroll_top(item_bottom - client_height);
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    view! {
        {move || {
            let items = suggestions.get();
            (!items.is_empty()).then(|| {
                let coords = cursor_coords.get();
                let selected = selected_idx.get();
                let current_item = items.get(selected).cloned();
                view! {
                    <div
                        class="suggestions-floating"
                        on:mousedown=move |e: web_sys::MouseEvent| { e.prevent_default(); }
                        style=format!("left:{}px; top:{}px", coords.0, coords.1)
                    >
                        <div class="suggestions-list-container">
                            {move || suggestions.get().into_iter().enumerate().map(|(i, s)| {
                                let s2 = s.clone();
                                let s3 = s.clone();
                                let (touch_start, set_touch_start) = signal((0.0, 0.0));
                                let (has_moved, set_has_moved) = signal(false);
                                view! {
                                    <button
                                        class=move || if selected_idx.get() == i { "suggestion-item selected" } else { "suggestion-item" }
                                        on:mousedown=move |e: web_sys::MouseEvent| {
                                            e.prevent_default();
                                            e.stop_propagation();
                                            on_select.run(s2.clone());
                                        }
                                        on:mouseup=move |e: web_sys::MouseEvent| {
                                            e.prevent_default();
                                            e.stop_propagation();
                                        }
                                        on:click=move |e: web_sys::MouseEvent| {
                                            e.prevent_default();
                                            e.stop_propagation();
                                        }
                                        on:touchstart=move |e: web_sys::TouchEvent| {
                                            if let Some(t) = e.touches().get(0) {
                                                set_touch_start.set((t.client_x() as f64, t.client_y() as f64));
                                                set_has_moved.set(false);
                                            }
                                        }
                                        on:touchmove=move |e: web_sys::TouchEvent| {
                                            if let Some(t) = e.touches().get(0) {
                                                let start = touch_start.get();
                                                let dx = t.client_x() as f64 - start.0;
                                                let dy = t.client_y() as f64 - start.1;
                                                let dist = (dx * dx + dy * dy).sqrt();
                                                if dist > 10.0 {
                                                    set_has_moved.set(true);
                                                }
                                            }
                                        }
                                        on:touchend=move |e: web_sys::TouchEvent| {
                                            if !has_moved.get() {
                                                e.prevent_default();
                                                on_select.run(s3.clone());
                                            }
                                        }
                                        on:mouseenter=move |_| selected_idx.set(i)
                                    >
                                        <span class="suggestion-kind">{kind_icon(s.kind)}</span>
                                        <span class="suggestion-label">{s.label.clone()}</span>
                                        {s.detail.map(|d| view! { <span class="suggestion-detail">{d}</span> })}
                                        {move || (selected_idx.get() == i && s.documentation.is_some()).then(|| {
                                            view! {
                                                <span
                                                    class="suggestion-read-more"
                                                    title="Toggle Details"
                                                    on:mousedown=move |e: web_sys::MouseEvent| {
                                                        e.prevent_default();
                                                        e.stop_propagation();
                                                    }
                                                    on:mouseup=move |e: web_sys::MouseEvent| {
                                                        e.prevent_default();
                                                        e.stop_propagation();
                                                    }
                                                    on:click=move |e: web_sys::MouseEvent| {
                                                        e.prevent_default();
                                                        e.stop_propagation();
                                                        show_docs.update(|v| *v = !*v);
                                                    }
                                                >
                                                    <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                                                        <circle cx="8" cy="8" r="6" />
                                                        <line x1="8" y1="8" x2="8" y2="11" />
                                                        <line x1="8" y1="5" x2="8" y2="5" />
                                                    </svg>
                                                </span>
                                            }
                                        })}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                        {move || (show_docs.get() && current_item.as_ref().and_then(|item| item.documentation.as_ref()).is_some()).then(|| {
                            let docs = current_item.as_ref().and_then(|item| item.documentation.as_ref()).unwrap().clone();
                            view! {
                                <div class="suggestion-docs">{docs}</div>
                            }
                        })}
                    </div>
                }
            })
        }}
    }
}
