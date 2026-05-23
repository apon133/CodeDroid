use leptos::prelude::*;

#[component]
pub fn LucideIcon(
    name: &'static str,
    #[prop(optional, default = "")] class: &'static str,
    #[prop(optional, default = "16")] size: &'static str,
) -> impl IntoView {
    let svg_content = match name {
        "folder" => view! {
            <path d="M20 20a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2 3h6a2 2 0 0 1 2 2z" />
        }.into_any(),
        "file" => view! {
            <path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" />
            <path d="M14 2v4a2 2 0 0 0 2 2h4" />
        }.into_any(),
        "search" => view! {
            <circle cx="11" cy="11" r="8" />
            <path d="m21 21-4.3-4.3" />
        }.into_any(),
        "package" => view! {
            <path d="M11 21.88a2 2 0 0 0 2 0l7.68-4.43a2 2 0 0 0 1-1.73V7.72a2 2 0 0 0-1-1.73L13 1.56a2 2 0 0 0-2 0L3.32 6a2 2 0 0 0-1 1.73v8a2 2 0 0 0 1 1.73Z" />
            <path d="M12 22V12" />
            <path d="m12 12 8.73-5.04" />
            <path d="m12 12-8.73-5.04" />
            <path d="M3.57 6.88 12 11.75l8.43-4.87" />
        }.into_any(),
        "save" => view! {
            <path d="M15.2 3a2 2 0 0 1 1.4.6l3.8 3.8a2 2 0 0 1 .6 1.4V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z" />
            <path d="M17 21v-7a1 1 0 0 0-1-1H8a1 1 0 0 0-1 1v7" />
            <path d="M7 3v4a1 1 0 0 0 1 1h7" />
        }.into_any(),
        "play" => view! {
            <polygon points="6 3 20 12 6 21 6 3" />
        }.into_any(),
        "stop" => view! {
            <rect width="18" height="18" x="3" y="3" rx="2" />
        }.into_any(),
        "copy" => view! {
            <rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
            <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
        }.into_any(),
        "clipboard" => view! {
            <rect width="8" height="4" x="8" y="2" rx="1" ry="1" />
            <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
        }.into_any(),

        "trash" => view! {
            <path d="M3 6h18" />
            <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
        }.into_any(),
        "arrow-left" => view! {
            <path d="m12 19-7-7 7-7" />
            <path d="M19 12H5" />
        }.into_any(),
        "chevron-right" => view! {
            <path d="m9 18 6-6-6-6" />
        }.into_any(),
        "chevron-down" => view! {
            <path d="m6 9 6 6 6-6" />
        }.into_any(),
        "plus" => view! {
            <path d="M5 12h14" />
            <path d="M12 5v14" />
        }.into_any(),
        "file-plus" => view! {
            <path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" />
            <path d="M14 2v4a2 2 0 0 0 2 2h4" />
            <path d="M9 15h6" />
            <path d="M12 12v6" />
        }.into_any(),
        "folder-plus" => view! {
            <path d="M20 20a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2 3h6a2 2 0 0 1 2 2Z" />
            <path d="M9 14h6" />
            <path d="M12 11v6" />
        }.into_any(),
        "terminal" => view! {
            <polyline points="4 17 10 11 4 5" />
            <line x1="12" x2="20" y1="19" y2="19" />
        }.into_any(),
        "settings" => view! {
            <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.1a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" /><circle cx="12" cy="12" r="3" />
        }.into_any(),
        "globe" => view! {
            <circle cx="12" cy="12" r="10" />
            <path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20" />
            <path d="M2 12h20" />
        }.into_any(),
        "menu" => view! {
            <line x1="4" x2="20" y1="12" y2="12" />
            <line x1="4" x2="20" y1="6" y2="6" />
            <line x1="4" x2="20" y1="18" y2="18" />
        }.into_any(),
        "code" => view! {
            <polyline points="16 18 22 12 16 6" />
            <polyline points="8 6 2 12 8 18" />
        }.into_any(),
        "x" => view! {
            <path d="M18 6 6 18" />
            <path d="m6 6 12 12" />
        }.into_any(),
        _ => view! {
            <circle cx="12" cy="12" r="10" />
        }.into_any()
    };

    view! {
        <svg
            class=class
            width=size
            height=size
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            stroke-linejoin="round"
            style="vertical-align: middle; flex-shrink: 0;"
        >
            {svg_content}
        </svg>
    }
}
