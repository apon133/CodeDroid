use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::icon::LucideIcon;
use crate::pages::editor::utils::{file_icon, FileEntry};
use crate::store;

#[derive(Clone, PartialEq)]
struct SearchResult {
    file: String,
    line_number: usize,
    column: usize,
    line: String,
    start_byte: usize,
    end_byte: usize,
}

fn find_matches(content: &str, query: &str) -> Vec<(usize, usize)> {
    find_matches_advanced(content, query, false, false, false)
}

fn glob_to_regex(pattern: &str) -> Option<regex::Regex> {
    let trimmed = pattern.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut regex_str = String::new();
    regex_str.push_str("(?i)"); // case-insensitive flag for the entire regex

    let has_wildcard = trimmed.contains('*') || trimmed.contains('?');
    if !has_wildcard {
        regex_str.push_str(".*");
    } else {
        regex_str.push('^');
    }

    for c in trimmed.chars() {
        match c {
            '*' => regex_str.push_str(".*"),
            '?' => regex_str.push('.'),
            '.' | '+' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '\\' | '|' => {
                regex_str.push('\\');
                regex_str.push(c);
            }
            _ => regex_str.push(c),
        }
    }

    if !has_wildcard {
        regex_str.push_str(".*");
    } else {
        regex_str.push('$');
    }

    regex::Regex::new(&regex_str).ok()
}

fn matches_filter(path: &str, filter_str: &str, default_match: bool) -> bool {
    let trimmed = filter_str.trim();
    if trimmed.is_empty() {
        return default_match;
    }

    let patterns: Vec<&str> = trimmed
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if patterns.is_empty() {
        return default_match;
    }

    for pattern in patterns {
        if let Some(re) = glob_to_regex(pattern) {
            if re.is_match(path) {
                return true;
            }
        }
    }

    false
}

fn is_whole_word(content: &str, start: usize, end: usize) -> bool {
    let before_char = if start > 0 {
        content[..start].chars().next_back()
    } else {
        None
    };
    let after_char = if end < content.len() {
        content[end..].chars().next()
    } else {
        None
    };

    let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

    let before_ok = before_char.map_or(true, |c| !is_word_char(c));
    let after_ok = after_char.map_or(true, |c| !is_word_char(c));

    before_ok && after_ok
}

fn find_matches_advanced(
    content: &str,
    query: &str,
    match_case: bool,
    whole_word: bool,
    use_regex: bool,
) -> Vec<(usize, usize)> {
    let needle = query.trim();
    if needle.is_empty() {
        return Vec::new();
    }

    if use_regex {
        let mut builder = regex::RegexBuilder::new(needle);
        builder.case_insensitive(!match_case);
        if let Ok(re) = builder.build() {
            let mut matches = Vec::new();
            for mat in re.find_iter(content) {
                let start = mat.start();
                let end = mat.end();

                if whole_word {
                    if is_whole_word(content, start, end) {
                        matches.push((start, end));
                    }
                } else {
                    matches.push((start, end));
                }

                if matches.len() >= 500 {
                    break;
                }
            }
            return matches;
        } else {
            return Vec::new();
        }
    }

    let search_str = if match_case {
        content.to_string()
    } else {
        content.to_lowercase()
    };
    let find_str = if match_case {
        needle.to_string()
    } else {
        needle.to_lowercase()
    };

    let mut matches = Vec::new();
    let mut start_pos = 0usize;

    while let Some(idx) = search_str[start_pos..].find(&find_str) {
        let actual_start = start_pos + idx;
        let actual_end = actual_start + find_str.len();

        if whole_word {
            if is_whole_word(content, actual_start, actual_end) {
                matches.push((actual_start, actual_end));
            }
        } else {
            matches.push((actual_start, actual_end));
        }

        start_pos = actual_start + find_str.len();
        if start_pos >= content.len() || matches.len() >= 500 {
            break;
        }
    }

    matches
}

fn byte_to_utf16_offset(content: &str, byte_offset: usize) -> u32 {
    let safe_offset = byte_offset.min(content.len());
    content[..safe_offset].encode_utf16().count() as u32
}

fn select_text_range(content: &str, start_byte: usize, end_byte: usize) {
    let start = byte_to_utf16_offset(content, start_byte);
    let end = byte_to_utf16_offset(content, end_byte);

    spawn_local(async move {
        gloo_timers::future::TimeoutFuture::new(25).await;
        use wasm_bindgen::JsCast;
        if let Some(target) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.query_selector(".code-editor").ok().flatten())
        {
            if let Ok(target) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                let _ = target.focus();
                let _ = target.set_selection_range(start, end);
            }
        }
    });
}

fn search_project_files(
    project_id: &str,
    files: Vec<FileEntry>,
    query: &str,
    match_case: bool,
    whole_word: bool,
    use_regex: bool,
    files_to_include: &str,
    files_to_exclude: &str,
) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let needle = query.trim();
    if needle.is_empty() {
        return results;
    }

    let include_str = files_to_include.trim();
    let exclude_str = files_to_exclude.trim();

    for entry in files.into_iter().filter(|entry| !entry.is_dir) {
        let show_file = if !include_str.is_empty() {
            matches_filter(&entry.name, include_str, true)
        } else {
            true
        };

        let is_excluded = if !exclude_str.is_empty() {
            matches_filter(&entry.name, exclude_str, false)
        } else {
            false
        };

        if !show_file || is_excluded {
            continue;
        }

        let key = store::file_key(project_id, &entry.name);
        let content = store::load_file(&key);
        for (start_byte, end_byte) in
            find_matches_advanced(&content, needle, match_case, whole_word, use_regex)
        {
            let prefix = &content[..start_byte];
            let line_number = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
            let line_start = prefix.rfind('\n').map(|idx| idx + 1).unwrap_or(0);
            let line_end = content[end_byte..]
                .find('\n')
                .map(|idx| end_byte + idx)
                .unwrap_or(content.len());
            let line = content[line_start..line_end].trim().to_string();

            results.push(SearchResult {
                file: entry.name.clone(),
                line_number,
                column: start_byte.saturating_sub(line_start),
                line,
                start_byte,
                end_byte,
            });

            if results.len() >= 200 {
                return results;
            }
        }
    }

    results
}

fn replace_all_advanced(
    content: &str,
    query: &str,
    replacement: &str,
    match_case: bool,
    whole_word: bool,
    use_regex: bool,
) -> (String, usize) {
    let matches = find_matches_advanced(content, query, match_case, whole_word, use_regex);
    if matches.is_empty() {
        return (content.to_string(), 0);
    }

    let mut output = String::with_capacity(content.len());
    let mut last = 0usize;
    for (start, end) in matches.iter().copied() {
        output.push_str(&content[last..start]);
        output.push_str(replacement);
        last = end;
    }
    output.push_str(&content[last..]);
    (output, matches.len())
}

#[component]
pub fn SearchBar(
    show_search: RwSignal<bool>,
    find_text: RwSignal<String>,
    find_index: RwSignal<usize>,
    code: RwSignal<String>,
) -> impl IntoView {
    let select_match = Callback::new(move |step: isize| {
        let content = code.get_untracked();
        let matches = find_matches(&content, &find_text.get_untracked());
        if matches.is_empty() {
            find_index.set(0);
            return;
        }

        let current = find_index.get_untracked();
        let next = if step < 0 {
            if current == 0 {
                matches.len() - 1
            } else {
                current - 1
            }
        } else {
            (current + 1) % matches.len()
        };
        find_index.set(next);
        let (start, end) = matches[next];
        select_text_range(&content, start, end);
    });

    view! {
        {move || show_search.get().then(|| view! {
            <div class="search-bar file-search-bar">
                <div class="file-search-input-wrap">
                    <LucideIcon name="search" size="16" />
                    <input class="input" type="search" placeholder="Search current file..."
                        prop:value=move || find_text.get()
                        on:input=move |e| {
                            find_text.set(event_target_value(&e));
                            find_index.set(0);
                            let content = code.get_untracked();
                            if let Some((start, end)) = find_matches(&content, &find_text.get_untracked()).first().copied() {
                                select_text_range(&content, start, end);
                            }
                        }
                        on:keydown=move |e: web_sys::KeyboardEvent| {
                            if e.key() == "Escape" {
                                show_search.set(false);
                            } else if e.key() == "Enter" {
                                e.prevent_default();
                                select_match.run(if e.shift_key() { -1 } else { 1 });
                            }
                        }
                    />
                    <span class="file-search-count">
                        {move || {
                            let total = find_matches(&code.get(), &find_text.get()).len();
                            if find_text.get().trim().is_empty() {
                                "0/0".to_string()
                            } else if total == 0 {
                                "0/0".to_string()
                            } else {
                                format!("{}/{}", find_index.get() + 1, total)
                            }
                        }}
                    </span>
                </div>
                <div class="file-search-actions">
                    <button class="btn btn-icon" title="Previous match" on:click=move |_| select_match.run(-1)>
                        <LucideIcon name="chevron-up" size="16" />
                    </button>
                    <button class="btn btn-icon" title="Next match" on:click=move |_| select_match.run(1)>
                        <LucideIcon name="chevron-down" size="16" />
                    </button>
                    <button class="btn btn-icon" title="Close search" on:click=move |_| show_search.set(false)>
                        <LucideIcon name="x" size="16" />
                    </button>
                </div>
            </div>
        })}
    }
}

#[component]
pub fn ProjectSearchReplacePanel(
    project_id: String,
    project_path: String,
    file_tree: Signal<Vec<FileEntry>>,
    file_tree_data: RwSignal<Vec<FileEntry>>,
    active_tab: RwSignal<Option<String>>,
    code: RwSignal<String>,
    dirty: RwSignal<bool>,
    project_query: RwSignal<String>,
    replace_text: RwSignal<String>,
    open_file: Callback<String>,
    trigger_diagnostics: Callback<String>,
    show_snack: Callback<String>,
    sidebar_open: Signal<bool>,
    close_sidebar: Callback<()>,
    _sidebar_mode: RwSignal<usize>,
) -> impl IntoView {
    let match_case = RwSignal::new(false);
    let whole_word = RwSignal::new(false);
    let use_regex = RwSignal::new(false);
    let files_to_include = RwSignal::new(String::new());
    let files_to_exclude = RwSignal::new(String::new());
    let show_details = RwSignal::new(false);

    let replace_all = Callback::new({
        let project_id = project_id.clone();
        let project_path = project_path.clone();
        move |_: ()| {
            let query = project_query.get_untracked();
            let replacement = replace_text.get_untracked();
            if query.trim().is_empty() {
                return;
            }

            let m_case = match_case.get_untracked();
            let w_word = whole_word.get_untracked();
            let u_regex = use_regex.get_untracked();
            let f_include = files_to_include.get_untracked();
            let f_exclude = files_to_exclude.get_untracked();

            let include_str = f_include.trim();
            let exclude_str = f_exclude.trim();

            let mut changed_files = Vec::<(String, String)>::new();
            let mut total_replacements = 0usize;
            for entry in file_tree
                .get_untracked()
                .into_iter()
                .filter(|entry| !entry.is_dir)
            {
                let show_file = if !include_str.is_empty() {
                    matches_filter(&entry.name, include_str, true)
                } else {
                    true
                };

                let is_excluded = if !exclude_str.is_empty() {
                    matches_filter(&entry.name, exclude_str, false)
                } else {
                    false
                };

                if !show_file || is_excluded {
                    continue;
                }

                let key = store::file_key(&project_id, &entry.name);
                let content = store::load_file(&key);
                let (updated, count) =
                    replace_all_advanced(&content, &query, &replacement, m_case, w_word, u_regex);
                if count > 0 {
                    store::save_file(&key, &updated);
                    total_replacements += count;
                    changed_files.push((entry.name, updated));
                }
            }

            if total_replacements == 0 {
                show_snack.run("No matches to replace.".to_string());
                return;
            }

            if let Some(active) = active_tab.get_untracked() {
                if let Some((_, updated)) = changed_files.iter().find(|(name, _)| *name == active) {
                    code.set(updated.clone());
                    dirty.set(true);
                    trigger_diagnostics.run(updated.clone());
                }
            }

            file_tree_data.set(crate::pages::editor::utils::build_file_tree(&project_id));

            for (name, updated) in changed_files {
                let full_path = format!("{}/{}", project_path, name);
                spawn_local(async move {
                    let _ = api::save_file_api(&full_path, &updated).await;
                });
            }

            show_snack.run(format!(
                "Replaced {} match{}.",
                total_replacements,
                if total_replacements == 1 { "" } else { "es" }
            ));
        }
    });

    view! {
        {move || sidebar_open.get().then(|| view! {
            <div class="sidebar-overlay" on:click=move |_| close_sidebar.run(()) />
        })}

        <div class=move || if sidebar_open.get() { "file-tree-panel search-replace-panel open" } else { "file-tree-panel search-replace-panel" }>


            <div class="search-panel-header">
                <div>
                    <div class="search-panel-title">"Search"</div>
                    <div class="search-panel-subtitle">"Find and replace across project"</div>
                </div>
                <button class="btn btn-icon mobile-only" title="Close search panel" on:click=move |_| close_sidebar.run(())>
                    <LucideIcon name="x" size="16" />
                </button>
            </div>

            <div class="search-panel-controls">
                <div class="search-panel-input">
                    <LucideIcon name="search" size="15" />
                    <input class="input" type="search" placeholder="Search..."
                        style="padding-right: 0;"
                        prop:value=move || project_query.get()
                        on:input=move |e| project_query.set(event_target_value(&e))
                    />
                    <div class="search-input-options" style="display:flex; gap:2px; flex-shrink:0;">
                        <button
                            class=move || if match_case.get() { "search-opt-btn active" } else { "search-opt-btn" }
                            title="Match Case"
                            on:click=move |_| match_case.update(|v| *v = !*v)
                        >
                            "Aa"
                        </button>
                        <button
                            class=move || if whole_word.get() { "search-opt-btn active" } else { "search-opt-btn" }
                            title="Match Whole Word"
                            on:click=move |_| whole_word.update(|v| *v = !*v)
                        >
                            "ab"
                        </button>
                        <button
                            class=move || if use_regex.get() { "search-opt-btn active" } else { "search-opt-btn" }
                            title="Use Regular Expression"
                            on:click=move |_| use_regex.update(|v| *v = !*v)
                        >
                            ".*"
                        </button>
                    </div>
                </div>
                <div class="search-panel-input">
                    <LucideIcon name="replace" size="15" />
                    <input class="input" type="text" placeholder="Replace..."
                        prop:value=move || replace_text.get()
                        on:input=move |e| replace_text.set(event_target_value(&e))
                    />
                    <button class="replace-all-btn" title="Replace all" on:click=move |_| replace_all.run(())>
                        <LucideIcon name="replace-all" size="15" />
                    </button>
                    <button
                        class=move || if show_details.get() { "replace-all-btn active" } else { "replace-all-btn" }
                        title="Toggle Search Details"
                        style="margin-left: 4px;"
                        on:click=move |_| show_details.update(|v| *v = !*v)
                    >
                        <LucideIcon name="more-horizontal" size="15" />
                    </button>
                </div>
            </div>

            {move || show_details.get().then(|| view! {
                <div class="search-panel-details" style="display:flex; flex-direction:column; gap:8px; padding: 0 12px 12px; border-bottom: 1px solid var(--border);">
                    <div style="display:flex; flex-direction:column; gap:4px;">
                        <span style="font-size:11px; color:var(--text2); font-weight:600;">"files to include"</span>
                        <div class="search-panel-input">
                            <LucideIcon name="folder-open" size="14" />
                            <input class="input" type="text" placeholder="e.g. src, *.rs"
                                prop:value=move || files_to_include.get()
                                on:input=move |e| files_to_include.set(event_target_value(&e))
                            />
                        </div>
                    </div>
                    <div style="display:flex; flex-direction:column; gap:4px;">
                        <span style="font-size:11px; color:var(--text2); font-weight:600;">"files to exclude"</span>
                        <div class="search-panel-input">
                            <LucideIcon name="settings" size="14" />
                            <input class="input" type="text" placeholder="e.g. target, node_modules"
                                prop:value=move || files_to_exclude.get()
                                on:input=move |e| files_to_exclude.set(event_target_value(&e))
                            />
                        </div>
                    </div>
                </div>
            })}

            {move || {
                let query = project_query.get();
                let m_case = match_case.get();
                let w_word = whole_word.get();
                let u_regex = use_regex.get();
                let f_include = files_to_include.get();
                let f_exclude = files_to_exclude.get();

                let results = search_project_files(
                    &project_id,
                    file_tree.get(),
                    &query,
                    m_case,
                    w_word,
                    u_regex,
                    &f_include,
                    &f_exclude,
                );
                let trimmed = query.trim().to_string();

                if trimmed.is_empty() {
                    return view! {
                        <div class="search-panel-empty">
                            "Search your project files from here."
                        </div>
                    }.into_any();
                }

                if results.is_empty() {
                    return view! {
                        <div class="search-panel-empty">
                            "No results found."
                        </div>
                    }.into_any();
                }

                view! {
                    <div class="search-panel-results">
                        <div class="search-panel-count">
                            {results.len()} " result" {if results.len() == 1 { "" } else { "s" }}
                            {if results.len() >= 200 { " shown" } else { "" }}
                        </div>
                        {results.into_iter().map(|result| {
                            let file_for_click = result.file.clone();
                            let file_for_icon = result.file.clone();
                            let file_for_display = result.file.clone();
                            let line_for_display = result.line_number;
                            let line_text = result.line.clone();
                            let start_byte = result.start_byte;
                            let end_byte = result.end_byte;
                            let project_id_for_click = project_id.clone();
                            let open_file_for_click = open_file;

                            view! {
                                <button class="search-panel-result" on:click=move |_| {
                                    open_file_for_click.run(file_for_click.clone());
                                    let key = store::file_key(&project_id_for_click, &file_for_click);
                                    let content = store::load_file(&key);
                                    select_text_range(&content, start_byte, end_byte);
                                }>
                                    <span class="search-panel-file">
                                        <img src=file_icon(&file_for_icon) alt="" />
                                        <span>{file_for_display}</span>
                                        <span class="search-panel-location">{format!(":{}", line_for_display)}</span>
                                    </span>
                                    <span class="search-panel-line">{line_text}</span>
                                </button>
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}
