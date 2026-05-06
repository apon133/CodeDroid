use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use web_sys::MouseEvent;

#[component]
pub fn AppBar(
    title: String,
    #[prop(optional)] back: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let navigate = use_navigate();
    let on_back = move |_: MouseEvent| { navigate("/", Default::default()); };

    view! {
        <div class="app-bar">
            <div class="app-bar-title">
                {if back {
                    view! {
                        <button class="btn btn-icon" on:click=on_back title="Back">
                            "←"
                        </button>
                    }.into_any()
                } else {
                    view! {
                        <span class="logo">"⟨/"</span>
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
