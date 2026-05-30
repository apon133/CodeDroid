use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

mod api;
mod components;
mod models;
mod pages;
mod store;

use pages::docs::DocsPage;
use pages::editor::EditorPage;
use pages::home::HomePage;
use pages::settings::SettingsPage;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div id="root">
                <Routes fallback=|| view! { <div class="home-empty"><p>"Page not found"</p></div> }>
                    <Route path=path!("/") view=HomePage />
                    <Route path=path!("/editor/:id") view=EditorPage />
                    <Route path=path!("/settings") view=SettingsPage />
                    <Route path=path!("/docs") view=DocsPage />
                </Routes>
            </div>
        </Router>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
