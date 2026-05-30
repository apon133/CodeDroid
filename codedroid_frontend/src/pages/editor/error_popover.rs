use leptos::prelude::*;
use crate::api;
use super::components::apply_replacement;

#[component]
pub fn ErrorPopover(
    cursor_coords: RwSignal<(f64, f64)>,
    active_error: RwSignal<Option<(api::Diagnostic, Vec<api::CodeSuggestion>, bool)>>,
    code: RwSignal<String>,
    show_snack: Callback<String>,
) -> impl IntoView {
    view! {
        {move || {
            if !active_error.get().is_some() {
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
    }
}
