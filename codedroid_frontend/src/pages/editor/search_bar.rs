use leptos::prelude::*;
use crate::components::icon::LucideIcon;

#[component]
pub fn SearchBar(
    show_search: RwSignal<bool>,
    find_text: RwSignal<String>,
) -> impl IntoView {
    view! {
        {move || show_search.get().then(|| view! {
            <div class="search-bar">
                <input class="input" type="text" placeholder="Find..."
                    prop:value=move || find_text.get()
                    on:input=move |e| find_text.set(event_target_value(&e))
                />
                <button class="btn btn-primary" style="padding:6px 12px;font-size:12px">"Find Next"</button>
                <button class="btn btn-icon" on:click=move |_| show_search.set(false)>
                    <LucideIcon name="x" size="16" />
                </button>
            </div>
        })}
    }
}
