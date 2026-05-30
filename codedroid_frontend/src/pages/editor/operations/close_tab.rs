use crate::store;
use leptos::prelude::*;

pub fn make_close_tab(
    pid: String,
    code: RwSignal<String>,
    active_tab: RwSignal<Option<String>>,
    open_tabs: RwSignal<Vec<String>>,
) -> Callback<String> {
    Callback::new(move |name: String| {
        open_tabs.update(|t| t.retain(|n| *n != name));
        let tabs = open_tabs.get_untracked();
        if active_tab.get_untracked().as_deref() == Some(&name) {
            if let Some(first) = tabs.first() {
                let key = store::file_key(&pid, first);
                code.set(store::load_file(&key));
                active_tab.set(Some(first.clone()));
            } else {
                active_tab.set(None);
                code.set(String::new());
            }
        }
    })
}
