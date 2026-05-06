use leptos::prelude::*;

/// A simple toast/snackbar component.
/// Pass a ReadSignal<Option<String>> — when Some(msg), it shows for 3s.
#[component]
pub fn Snackbar(message: ReadSignal<Option<String>>) -> impl IntoView {
    let visible = move || message.get().is_some();
    let text   = move || message.get().unwrap_or_default();

    view! {
        <div
            class=move || if visible() { "snackbar show" } else { "snackbar" }
        >
            {text}
        </div>
    }
}
