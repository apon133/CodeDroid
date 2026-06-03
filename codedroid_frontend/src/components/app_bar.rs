use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use web_sys::MouseEvent;

use crate::components::icon::LucideIcon;

#[component]
pub fn AppBar(
    title: String,
    #[prop(optional)] back: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let navigate = use_navigate();
    let on_back = move |_: MouseEvent| {
        let history = web_sys::window().and_then(|w| w.history().ok());
        if let Some(history) = history {
            if history.length().unwrap_or(0) > 1 {
                if history.back().is_ok() {
                    return;
                }
            }
        }
        navigate("/", Default::default());
    };

    view! {
        <div class="app-bar">
            <div class="app-bar-title">
                {if back {
                    view! {
                        <button class="btn btn-icon" on:click=on_back title="Back">
                            <LucideIcon name="arrow-left" size="20" />
                        </button>
                    }.into_any()
                } else {
                    view! {
                        <span class="logo">
                            <LucideIcon name="code" size="22" />
                        </span>
                    }.into_any()
                }}
                <span>{title}</span>
            </div>
            <div class="app-bar-actions">
                {children.map(|c| c())}
            </div>
        </div>
    }
}
