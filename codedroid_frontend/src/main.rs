use leptos::prelude::*;
use leptos_router::components::{Router, Route, Routes};
use leptos_router::path;

mod models;
mod store;
mod pages;
mod components;
mod api;

use pages::home::HomePage;
use pages::editor::EditorPage;
use pages::settings::SettingsPage;
use pages::docs::DocsPage;

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
