use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use web_sys;

thread_local! {
    pub static SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    pub static THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

pub fn highlight_code(code: &str, ext: &str) -> String {
    let mapped_ext = match ext {
        "dart" | "kt" => "java",
        "ts" | "tsx" | "jsx" => "js",
        "swift" => "cs",
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

pub fn file_extension(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or("")
}

pub fn file_icon(name: &str) -> &'static str {
    match file_extension(name) {
        "rs"   => "🦀", "go"   => "🐹", "py"   => "🐍",
        "js" | "ts" | "jsx" | "tsx" => "⚡",
        "java" => "☕", "dart" => "🎯", "c" | "cpp" | "h" | "hpp" => "⚙️",
        "cs"   => "🔷", "kt"   => "🟣", "swift" => "🍎", "rb"   => "💎",
        "html" => "🌐", "css"  => "🎨", "toml" | "yaml" | "json" => "📋",
        _      => "📄",
    }
}

pub fn kind_icon(kind: Option<u32>) -> &'static str {
    match kind {
        Some(1) => "📝", // Text
        Some(2) | Some(3) => "𝑓", // Method/Function
        Some(4) => "🏗", // Constructor
        Some(5) => "🏷", // Field
        Some(6) => "𝑥", // Variable
        Some(7) => "📦", // Class
        Some(8) => "📜", // Interface
        Some(9) => "📦", // Module
        Some(10) => "🔧", // Property
        Some(11) => "📏", // Unit
        Some(12) => "🔢", // Value
        Some(13) => "🎨", // Enum
        Some(14) => "🔑", // Keyword
        Some(15) => "⌨", // Snippet
        Some(16) => "🎨", // Color
        Some(17) => "📄", // File
        Some(18) => "🔗", // Reference
        Some(19) => "📁", // Folder
        Some(20) => "🎨", // EnumMember
        Some(21) => "🧱", // Constant
        Some(22) => "🏗", // Struct
        Some(23) => "📅", // Event
        Some(24) => "⚙", // Operator
        Some(25) => "🧩", // TypeParameter
        _ => "📄",
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
    let mut entries: Vec<FileEntry> = Vec::new();

    for i in 0..len {
        if let Ok(Some(k)) = storage.key(i) {
            if let Some(rel) = k.strip_prefix(&prefix) {
                entries.push(FileEntry {
                    name: rel.to_string(),
                    key: k.clone(),
                    is_dir: false,
                });
            }
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}
