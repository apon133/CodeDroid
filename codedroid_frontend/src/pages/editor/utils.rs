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

pub fn is_project_source_file(filename: &str, lang: &str) -> bool {
    let ext = file_extension(filename).to_lowercase();
    match lang.to_lowercase().as_str() {
        "rust" => ext == "rs",
        "python" => ext == "py",
        "go" => ext == "go",
        "javascript" => ext == "js" || ext == "jsx",
        "typescript" => ext == "ts" || ext == "tsx",
        "c" => ext == "c" || ext == "h",
        "cpp" => ext == "cpp" || ext == "cc" || ext == "h" || ext == "hpp",
        "java" => ext == "java",
        "dart" => ext == "dart",
        "ruby" => ext == "rb",
        "kotlin" => ext == "kt",
        "swift" => ext == "swift",
        _ => false,
    }
}

pub fn file_lang_name(name: &str) -> &'static str {
    if name == "Cargo.toml" { return "Rust Config"; }
    if name == "go.mod" { return "Go Config"; }
    if name == "package.json" { return "Node Config"; }
    if name == "pubspec.yaml" { return "Dart Config"; }
    
    match file_extension(name) {
        "rs"   => "Rust",
        "go"   => "Go",
        "py"   => "Python",
        "js"   => "JavaScript",
        "ts"   => "TypeScript",
        "jsx"  => "React JS",
        "tsx"  => "React TS",
        "java" => "Java",
        "dart" => "Dart",
        "c"    => "C",
        "cpp"  => "C++",
        "h" | "hpp" => "Header",
        "cs"   => "C#",
        "kt"   => "Kotlin",
        "swift"=> "Swift",
        "rb"   => "Ruby",
        "html" => "HTML",
        "css"  => "CSS",
        "toml" => "TOML",
        "yaml" | "yml" => "YAML",
        "json" => "JSON",
        "md"   => "Markdown",
        "sh"   => "Shell",
        _      => "Text",
    }
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
