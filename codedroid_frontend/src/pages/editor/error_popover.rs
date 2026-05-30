use super::components::apply_replacement;
use crate::api;
use leptos::prelude::*;

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

                let severity = diag.severity.unwrap_or(1);
                let popover_class = match severity {
                    1 => "error-floating-popover",
                    2 => "error-floating-popover warning",
                    3 => "error-floating-popover info",
                    4 => "error-floating-popover hint",
                    _ => "error-floating-popover",
                };
                let title_class = match severity {
                    1 => "error-floating-title",
                    2 => "error-floating-title warning",
                    3 => "error-floating-title info",
                    4 => "error-floating-title hint",
                    _ => "error-floating-title",
                };
                let severity_icon = match severity {
                    1 => "🔴",
                    2 => "🟡",
                    3 => "🔵",
                    4 => "💡",
                    _ => "🔴",
                };

                view! {
                    <div
                        class=popover_class
                        style=format!("left:{}px; top:{}px", coords.0, coords.1)
                        on:mousedown=move |e: web_sys::MouseEvent| {
                            e.prevent_default();
                            e.stop_propagation();
                        }
                        on:click=move |e: web_sys::MouseEvent| {
                            e.prevent_default();
                            e.stop_propagation();
                        }
                    >
                        <div class="error-floating-header">
                            <span class="error-floating-icon">{severity_icon}</span>
                            <span class=title_class>{diag.message}</span>
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
                                            let explanation = sugg.explanation.clone();
                                            let replacement = sugg.replacement.clone();
                                            let range = sugg.range.clone();
                                            let snack_cb = snack;
                                            let code_cb = code_sig;
                                            let active_error_cb = active_error_sig;

                                            let has_fix = replacement.is_some() && range.is_some();
                                            let show_expl = RwSignal::new(false);

                                            let on_apply = move |e: web_sys::MouseEvent| {
                                                e.stop_propagation();
                                                if let (Some(repl), Some(r)) = (&replacement, &range) {
                                                    let orig = code_cb.get_untracked();
                                                    let updated = apply_replacement(&orig, r, repl);
                                                    code_cb.set(updated);
                                                    snack_cb.run("Quick Fix applied successfully!".to_string());
                                                    active_error_cb.set(None);
                                                }
                                            };

                                            let toggle_expl = move |_| {
                                                show_expl.update(|v| *v = !*v);
                                            };

                                            view! {
                                                <div class="error-floating-suggestion-wrapper" style="display:flex; flex-direction:column;">
                                                    <div
                                                        class="error-floating-suggestion-item"
                                                        on:click=toggle_expl
                                                        style="cursor:pointer;"
                                                    >
                                                        <span class="lightbulb-icon">"💡"</span>
                                                        <span class="suggestion-text">{title}</span>
                                                        {has_fix.then(|| view! {
                                                            <button class="btn btn-primary btn-xs" on:click=on_apply style="margin-left:auto;padding:2px 6px;font-size:10px">
                                                                "Fix"
                                                            </button>
                                                        })}
                                                    </div>
                                                    {move || show_expl.get().then(|| view! {
                                                        <div class="error-floating-suggestion-explanation" style="padding: 4px 8px 6px 22px; font-size: 10px; color: #a5a5a5; line-height: 1.4; border-top: 1px solid rgba(255,255,255,0.03);">
                                                            {explanation.clone()}
                                                        </div>
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
