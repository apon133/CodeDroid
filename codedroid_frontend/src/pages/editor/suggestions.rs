use leptos::prelude::*;
use crate::api;
use super::utils::kind_icon;

#[component]
pub fn SuggestionsOverlay(
    cursor_coords: RwSignal<(f64, f64)>,
    suggestions: RwSignal<Vec<api::CompletionItem>>,
    selected_idx: RwSignal<usize>,
    on_select: Callback<api::CompletionItem>,
) -> impl IntoView {
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
            })
        }}
    }
}
