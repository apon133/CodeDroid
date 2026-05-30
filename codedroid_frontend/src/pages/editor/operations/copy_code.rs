use leptos::prelude::*;

pub fn make_copy_code(
    code: RwSignal<String>,
    show_snack: Callback<String>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        let c = code.get_untracked();
        if let Some(window) = web_sys::window() {
            let _ = window.navigator().clipboard().write_text(&c);
            show_snack.run("Code copied!".to_string());
        }
    })
}
