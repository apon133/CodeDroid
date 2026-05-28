use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::html::{highlighted_html_for_string, styled_line_to_highlighted_html, IncludeBackground};
use syntect::easy::HighlightLines;
use web_sys;
use leptos::prelude::*;

thread_local! {
    pub static SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    pub static THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

#[allow(dead_code)]
pub fn highlight_code(code: &str, ext: &str) -> String {
    let normalized = ext.to_lowercase();
    let mapped_ext = match normalized.as_str() {
        "rust" => "rs",
        "python" => "py",
        "javascript" | "js" => "js",
        "typescript" | "ts" => "ts",
        "golang" | "go" => "go",
        "c" => "c",
        "cpp" | "c++" => "cpp",
        "ruby" | "rb" => "rb",
        "java" => "java",
        "dart" | "kt" | "kotlin" => "java",
        "tsx" | "jsx" => "js",
        "swift" => "cs",
        "vue" | "svelte" => "html",
        _ => ext,
    };

    SYNTAX_SET.with(|ss| {
        THEME_SET.with(|ts| {
            let syntax = ss.find_syntax_by_extension(mapped_ext)
                .unwrap_or_else(|| ss.find_syntax_plain_text());
            let theme = &ts.themes["base16-ocean.dark"];
            highlighted_html_for_string(code, ss, syntax, theme).unwrap_or_else(|_| code.to_string())
        })
    })
}

pub fn highlight_code_lines(code: &str, ext: &str) -> Vec<String> {
    let mapped_ext = match ext {
        "dart" | "kt" => "java",
        "ts" | "tsx" | "jsx" => "js",
        "swift" => "cs",
        "vue" | "svelte" => "html",
        _ => ext,
    };

    SYNTAX_SET.with(|ss| {
        THEME_SET.with(|ts| {
            let syntax = ss.find_syntax_by_extension(mapped_ext)
                .unwrap_or_else(|| ss.find_syntax_plain_text());
            let theme = &ts.themes["base16-ocean.dark"];
            
            let mut highlighter = HighlightLines::new(syntax, theme);
            let mut lines = Vec::new();
            
            for line in code.split('\n') {
                let line_with_nl = format!("{}\n", line);
                let regions = highlighter.highlight_line(&line_with_nl, ss).unwrap_or_default();
                let html = styled_line_to_highlighted_html(&regions, IncludeBackground::No).unwrap_or_default();
                lines.push(html);
            }
            lines
        })
    })
}

pub fn file_extension(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or("")
}

pub fn file_to_lsp_lang(filename: &str) -> String {
    match file_extension(filename).to_lowercase().as_str() {
        "rs" => "rust".to_string(),
        "py" => "python".to_string(),
        "js" => "javascript".to_string(),
        "jsx" => "jsx".to_string(),
        "ts" => "typescript".to_string(),
        "tsx" => "tsx".to_string(),
        "go" => "go".to_string(),
        "c" | "h" => "c".to_string(),
        "cpp" | "hpp" | "cc" => "cpp".to_string(),
        "java" => "java".to_string(),
        "dart" => "dart".to_string(),
        "rb" => "ruby".to_string(),
        "kt" => "kotlin".to_string(),
        "swift" => "swift".to_string(),
        "html" | "htm" => "html".to_string(),
        "css" => "css".to_string(),
        "vue" => "vue".to_string(),
        "svelte" => "svelte".to_string(),
        _ => "text".to_string(),
    }
}

#[allow(dead_code)]
fn is_web_lang(lang: &str) -> bool {
    let l = lang.to_lowercase();
    l == "javascript" || l == "typescript" || l == "html" || l == "css" || l == "vue" || l == "svelte" ||
    l == "react" || l == "nextjs" || l == "next.js" || l == "remix" || l == "angular" || l == "vanilla" ||
    l == "jsx" || l == "tsx"
}

pub fn is_project_source_file(filename: &str, _lang: &str) -> bool {
    let ext = file_extension(filename).to_lowercase();
    matches!(ext.as_str(), 
        "rs" | "py" | "js" | "jsx" | "ts" | "tsx" | "go" | "c" | "h" | "cpp" | "hpp" | "cc" | "java" | "dart" | "rb" | "kt" | "swift" | "html" | "htm" | "css" | "vue" | "svelte"
    )
}

pub fn file_lang_name(name: &str) -> &'static str {
    let lower_name = name.to_lowercase();
    if lower_name == "cargo.toml" { return "Rust Config"; }
    if lower_name == "go.mod" { return "Go Config"; }
    if lower_name == "package.json" { return "Node Config"; }
    if lower_name == "pubspec.yaml" { return "Dart Config"; }
    if lower_name == "gemfile" || lower_name == "gemfile.lock" { return "Ruby Config"; }
    if lower_name == "requirements.txt" || lower_name == "pipfile" || lower_name == "pyproject.toml" { return "Python Config"; }
    if lower_name == "build.gradle" || lower_name == "pom.xml" { return "Java Config"; }
    if lower_name == "composer.json" { return "PHP Config"; }
    
    match file_extension(name).to_lowercase().as_str() {
        "rs"   => "Rust",
        "go"   => "Go",
        "py"   => "Python",
        "js"   => "JavaScript",
        "ts"   => "TypeScript",
        "jsx"  => "React JS",
        "tsx"  => "React TS",
        "vue"  => "Vue",
        "svelte" => "Svelte",
        "java" => "Java",
        "dart" => "Dart",
        "c"    => "C",
        "cpp"  => "C++",
        "h" | "hpp" => "Header",
        "cs"   => "C#",
        "csproj" => "NuGet Project",
        "sln"  => "Visual Studio Solution",
        "kt"   => "Kotlin",
        "swift"=> "Swift",
        "rb"   => "Ruby",
        "html" => "HTML",
        "css"  => "CSS",
        "toml" => "TOML",
        "yaml" | "yml" => "YAML",
        "json" => "JSON",
        "md" | "markdown" => "Markdown",
        "sh" | "bash" => "Shell",
        _      => "Text",
    }
}

pub fn file_icon(name: &str) -> &'static str {
    let lower_name = name.to_lowercase();
    if lower_name == "cargo.toml" || lower_name == "cargo.lock" { return "/assets/icons/cargo.svg"; }
    if lower_name == "go.mod" || lower_name == "go.sum" || lower_name == "go.work" { return "/assets/icons/gomod.svg"; }
    if lower_name == "package.json" || lower_name == "package-lock.json" || lower_name == "yarn.lock" || lower_name == "pnpm-lock.yaml" { return "/assets/icons/npm.svg"; }
    if lower_name == "pubspec.yaml" || lower_name == "pubspec.lock" { return "/assets/icons/yaml.svg"; }
    if lower_name == "requirements.txt" || lower_name == "pipfile" || lower_name == "pipfile.lock" || lower_name == "pyproject.toml" || lower_name == "setup.py" { return "/assets/icons/python.svg"; }
    if lower_name == "build.gradle" || lower_name == "build.gradle.kts" || lower_name == "settings.gradle" || lower_name == "settings.gradle.kts" || lower_name == "gradle.properties" { return "/assets/icons/gradle.svg"; }
    if lower_name == "pom.xml" { return "/assets/icons/maven.svg"; }
    if lower_name == "composer.json" || lower_name == "composer.lock" { return "/assets/icons/composer.svg"; }
    if lower_name == "gemfile" || lower_name == "gemfile.lock" { return "/assets/icons/ruby.svg"; }
    if lower_name == "nuxt.config.js" || lower_name == "nuxt.config.ts" { return "/assets/icons/nuxt.svg"; }
    if lower_name == "next.config.js" || lower_name == "next.config.mjs" || lower_name == "next.config.ts" { return "/assets/icons/nextjs.svg"; }
    if lower_name == "angular.json" { return "/assets/icons/angular.svg"; }

    match file_extension(name).to_lowercase().as_str() {
        "rs"   => "/assets/icons/rust.svg",
        "go"   => "/assets/icons/go.svg",
        "py"   => "/assets/icons/python.svg",
        "js"   => "/assets/icons/javascript.svg",
        "jsx"  => "/assets/icons/react.svg",
        "ts"   => "/assets/icons/typescript.svg",
        "tsx"  => "/assets/icons/react.svg",
        "vue"  => "/assets/icons/vue.svg",
        "svelte" => "/assets/icons/svelte.svg",
        "java" => "/assets/icons/java.svg",
        "dart" => "/assets/icons/dart.svg",
        "c" | "h" => "/assets/icons/c.svg",
        "cpp" | "hpp" | "cc" => "/assets/icons/cpp.svg",
        "cs"   => "/assets/icons/csharp.svg",
        "csproj" | "sln" => "/assets/icons/nuget.svg",
        "kt"   => "/assets/icons/kotlin.svg",
        "swift"=> "/assets/icons/swift.svg",
        "rb"   => "/assets/icons/ruby.svg",
        "yaml" | "yml" => "/assets/icons/yaml.svg",
        "toml" => "/assets/icons/toml.svg",
        "json" => "/assets/icons/json.svg",
        "md" | "markdown" => "/assets/icons/markdown.svg",
        "html" => "/assets/icons/html.svg",
        "css"  => "/assets/icons/css.svg",
        "sh" | "bash" => "/assets/icons/shell.svg",
        _      => "/assets/icons/generic.svg",
    }
}

pub fn kind_icon(kind: Option<u32>) -> impl IntoView {
    match kind {
        Some(1) => view! {
            // Text
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#94a3b8" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <rect x="3" y="3" width="10" height="10" rx="1.5"/>
                <line x1="6" y1="6" x2="10" y2="6"/>
                <line x1="6" y1="9" x2="9" y2="9"/>
            </svg>
        }.into_any(),
        Some(2) | Some(3) => view! {
            // Method / Function
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#f1f5f9" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <rect x="2" y="2" width="12" height="4" rx="1.5" />
                <rect x="2" y="9" width="4" height="4" rx="1" />
                <rect x="10" y="9" width="4" height="4" rx="1" />
            </svg>
        }.into_any(),
        Some(4) => view! {
            // Constructor
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#fbbf24" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <rect x="2" y="2" width="12" height="4" rx="1.5" />
                <rect x="2" y="9" width="4" height="4" rx="1" />
                <rect x="10" y="9" width="4" height="4" rx="1" />
            </svg>
        }.into_any(),
        Some(5) => view! {
            // Field
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#38bdf8" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M2 8 L8 8" />
                <circle cx="11" cy="8" r="3" />
            </svg>
        }.into_any(),
        Some(6) => view! {
            // Variable
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#38bdf8" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M2 8 L8 8" />
                <circle cx="11" cy="8" r="3" />
            </svg>
        }.into_any(),
        Some(7) => view! {
            // Class
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#a855f7" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M8 2 L13 5 L13 11 L8 14 L3 11 L3 5 Z" />
                <path d="M8 8 L8 14" />
                <path d="M8 8 L3 5" />
                <path d="M8 8 L13 5" />
            </svg>
        }.into_any(),
        Some(8) => view! {
            // Interface
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#22d3ee" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M8 2 L13 5 L13 11 L8 14 L3 11 L3 5 Z" />
                <path d="M8 8 L8 14" />
                <path d="M8 8 L3 5" />
                <path d="M8 8 L13 5" />
            </svg>
        }.into_any(),
        Some(9) => view! {
            // Module
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#3b82f6" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M2 4.5 L8 2.5 L14 4.5 L14 11.5 L8 13.5 L2 11.5 Z" />
                <path d="M2 4.5 L8 6.5 L14 4.5" />
                <path d="M8 6.5 L8 13.5" />
            </svg>
        }.into_any(),
        Some(10) => view! {
            // Property
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#a855f7" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <rect x="2" y="2" width="12" height="4" rx="1.5" />
                <rect x="2" y="9" width="4" height="4" rx="1" />
                <rect x="10" y="9" width="4" height="4" rx="1" />
            </svg>
        }.into_any(),
        Some(11) => view! {
            // Unit
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#94a3b8" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <rect x="3" y="3" width="10" height="10" rx="1.5"/>
                <line x1="6" y1="6" x2="10" y2="6"/>
            </svg>
        }.into_any(),
        Some(12) => view! {
            // Value
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#38bdf8" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="8" cy="8" r="6" />
                <path d="M8 5 L8 11" />
            </svg>
        }.into_any(),
        Some(13) => view! {
            // Enum
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#fbbf24" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="4" cy="4" r="2" />
                <circle cx="12" cy="4" r="2" />
                <circle cx="4" cy="12" r="2" />
                <circle cx="12" cy="12" r="2" />
                <path d="M6 4 L10 4 M6 12 L10 12 M4 6 L4 10 M12 6 L12 10" />
            </svg>
        }.into_any(),
        Some(14) => view! {
            // Keyword
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#f43f5e" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="5" cy="11" r="3"/>
                <path d="M7.5 8.5 L13 3 L14.5 4.5 M10.5 6 L12 7.5"/>
            </svg>
        }.into_any(),
        Some(15) => view! {
            // Snippet
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#10b981" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M5 3 C3.5 3 3 4 3 6 C3 8 2 8.5 2 8.5 C2 8.5 3 9 3 11 C3 13 3.5 14 5 14 M11 3 C12.5 3 13 4 13 6 C13 8 14 8.5 14 8.5 C14 8.5 13 9 13 11 C13 13 12.5 14 11 14"/>
            </svg>
        }.into_any(),
        Some(16) => view! {
            // Color
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#ec4899" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="8" cy="8" r="6" fill="#ec4899"/>
            </svg>
        }.into_any(),
        Some(17) => view! {
            // File
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#94a3b8" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M3 2 L10 2 L13 5 L13 14 L3 14 Z" />
                <path d="M10 2 L10 5 L13 5" />
            </svg>
        }.into_any(),
        Some(18) => view! {
            // Reference
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#38bdf8" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M2 8 L8 8" />
                <circle cx="11" cy="8" r="3" />
            </svg>
        }.into_any(),
        Some(19) => view! {
            // Folder
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#fbbf24" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M2 3 L6 3 L8 5 L14 5 L14 13 L2 13 Z" />
            </svg>
        }.into_any(),
        Some(20) => view! {
            // EnumMember
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#fbbf24" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="8" cy="8" r="3" fill="#fbbf24" />
            </svg>
        }.into_any(),
        Some(21) => view! {
            // Constant
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#10b981" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <rect x="3" y="3" width="10" height="10" rx="1.5" />
            </svg>
        }.into_any(),
        Some(22) => view! {
            // Struct
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#e2e8f0" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M8 2 L13 5 L13 11 L8 14 L3 11 L3 5 Z" />
                <path d="M8 8 L8 14" />
                <path d="M8 8 L3 5" />
                <path d="M8 8 L13 5" />
            </svg>
        }.into_any(),
        Some(23) => view! {
            // Event
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#e11d48" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M8 2 L3 9 L8 9 L6 14 L13 7 L8 7 Z" />
            </svg>
        }.into_any(),
        Some(24) => view! {
            // Operator
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#94a3b8" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="8" cy="8" r="6" />
                <path d="M6 8 L10 8 M8 6 L8 10" />
            </svg>
        }.into_any(),
        Some(25) => view! {
            // TypeParameter
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#10b981" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M3 5 L6 8 L3 11 M13 5 L10 8 L13 11" />
            </svg>
        }.into_any(),
        _ => view! {
            // Default File
            <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="#94a3b8" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M3 2 L10 2 L13 5 L13 14 L3 14 Z" />
                <path d="M10 2 L10 5 L13 5" />
            </svg>
        }.into_any(),
    }
}

#[derive(Clone, PartialEq)]
pub struct FileEntry {
    pub name: String,
    pub key: String,
    pub is_dir: bool,
}

pub fn build_file_tree(project_id: &str) -> Vec<FileEntry> {
    let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    let len = storage.length().unwrap_or(0);
    let prefix = format!("codedroid_file_{}_", project_id);
    let mut files = std::collections::HashSet::new();
    let mut dirs = std::collections::HashSet::new();

    for i in 0..len {
        if let Ok(Some(k)) = storage.key(i) {
            if let Some(rel) = k.strip_prefix(&prefix) {
                if rel.ends_with("/.codedroid_dir") {
                    let dir_name = rel.trim_end_matches("/.codedroid_dir");
                    if !dir_name.is_empty() {
                        dirs.insert(dir_name.to_string());
                    }
                } else if !rel.is_empty() {
                    files.insert(rel.to_string());
                    // Add implicit parent directories
                    let mut parts: Vec<&str> = rel.split('/').collect();
                    while parts.len() > 1 {
                        parts.pop();
                        let dir = parts.join("/");
                        if !dir.is_empty() {
                            dirs.insert(dir);
                        }
                    }
                }
            }
        }
    }

    let mut entries: Vec<FileEntry> = Vec::new();
    for d in dirs {
        entries.push(FileEntry {
            name: d.clone(),
            key: format!("{}{}/.codedroid_dir", prefix, d),
            is_dir: true,
        });
    }
    for f in files {
        if f != ".codedroid_dir" && !f.ends_with("/.codedroid_dir") {
            entries.push(FileEntry {
                name: f.clone(),
                key: format!("{}{}", prefix, f),
                is_dir: false,
            });
        }
    }

    // Sort hierarchically: directories first at each level, then files, sorted alphabetically
    entries.sort_by(compare_hierarchical);
    entries
}

pub fn compare_hierarchical(a: &FileEntry, b: &FileEntry) -> std::cmp::Ordering {
    let a_parts: Vec<&str> = a.name.split('/').collect();
    let b_parts: Vec<&str> = b.name.split('/').collect();
    
    let min_len = std::cmp::min(a_parts.len(), b_parts.len());
    for i in 0..min_len {
        if a_parts[i] != b_parts[i] {
            let a_is_dir = i < a_parts.len() - 1 || (i == a_parts.len() - 1 && a.is_dir);
            let b_is_dir = i < b_parts.len() - 1 || (i == b_parts.len() - 1 && b.is_dir);
            
            if a_is_dir != b_is_dir {
                if a_is_dir {
                    return std::cmp::Ordering::Less;
                } else {
                    return std::cmp::Ordering::Greater;
                }
            }
            return a_parts[i].cmp(b_parts[i]);
        }
    }
    
    a_parts.len().cmp(&b_parts.len())
}


pub fn path_basename(path: &str) -> &str {
    path.split('/').last().unwrap_or(path)
}

pub fn path_depth(path: &str) -> usize {
    if path.is_empty() {
        return 0;
    }
    path.split('/').count() - 1
}

pub fn get_ancestors(path: &str) -> Vec<String> {
    let mut ancestors = Vec::new();
    let parts: Vec<&str> = path.split('/').collect();
    let mut current = String::new();
    for i in 0..(parts.len().saturating_sub(1)) {
        if i > 0 {
            current.push('/');
        }
        current.push_str(parts[i]);
        ancestors.push(current.clone());
    }
    ancestors
}

pub fn pos_to_index(code: &str, line: u32, character: u32) -> u32 {
    let mut current_idx = 0;
    for (i, l) in code.lines().enumerate() {
        if i as u32 == line {
            let chars: Vec<char> = l.chars().collect();
            let char_offset = std::cmp::min(character as usize, chars.len());
            let offset_str: String = chars[..char_offset].iter().collect();
            return current_idx + offset_str.encode_utf16().count() as u32;
        }
        current_idx += (l.encode_utf16().count() + 1) as u32;
    }
    current_idx
}

pub fn resolve_completion(item: &crate::api::CompletionItem) -> (String, Option<usize>) {
    if let Some(ref raw_snippet) = item.insert_text {
        let mut result = String::new();
        let mut cursor_offset = None;
        let chars: Vec<char> = raw_snippet.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            if chars[i] == '$' && i + 1 < chars.len() {
                let next = chars[i + 1];
                if next.is_ascii_digit() {
                    let is_primary = next == '0' || next == '1';
                    if is_primary && cursor_offset.is_none() {
                        cursor_offset = Some(result.encode_utf16().count());
                    }
                    i += 2;
                } else if next == '{' {
                    let mut j = i + 2;
                    let mut content = String::new();
                    while j < chars.len() && chars[j] != '}' {
                        content.push(chars[j]);
                        j += 1;
                    }
                    if j < chars.len() {
                        let placeholder = if let Some(colon_pos) = content.find(':') {
                            &content[colon_pos + 1..]
                        } else {
                            ""
                        };
                        if cursor_offset.is_none() {
                            cursor_offset = Some(result.encode_utf16().count());
                        }
                        result.push_str(placeholder);
                        i = j + 1;
                    } else {
                        result.push('$');
                        i += 1;
                    }
                } else {
                    result.push('$');
                    i += 1;
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        (result, cursor_offset)
    } else {
        let label = &item.label;
        if let Some(pos) = label.find("(...)") {
            let cleaned = label.replace("(...)", "()");
            (cleaned, Some(pos + 1))
        } else if let Some(pos) = label.find("{...}") {
            let cleaned = label.replace("{...}", "{}");
            (cleaned, Some(pos + 1))
        } else if let Some(pos) = label.find("[...]") {
            let cleaned = label.replace("[...]", "[]");
            (cleaned, Some(pos + 1))
        } else {
            (label.clone(), None)
        }
    }
}
