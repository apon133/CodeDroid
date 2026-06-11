/// install_language.rs
/// Handles language + LSP installation inside the Alpine Linux PRoot
/// environment running in Termux, and also directly via Termux's pkg.
use axum::Json;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Deserialize)]
pub struct InstallLanguageRequest {
    pub language: String,
}

#[derive(Serialize)]
pub struct InstallLanguageResponse {
    pub success: bool,
    pub message: String,
    pub output: String,
}

#[derive(Deserialize)]
pub struct CheckLanguageRequest {
    pub language: String,
}

#[derive(Serialize)]
pub struct CheckLanguageResponse {
    pub installed: bool,
    pub language: String,
}

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

fn termux_prefix() -> String {
    std::env::var("PREFIX").unwrap_or_else(|_| "/data/data/com.termux/files/usr".to_string())
}

fn termux_home() -> String {
    std::env::var("TERMUX_HOME")
        .or_else(|_| std::env::var("CODEDROID_FILES_DIR"))
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| "/data/data/com.termux/files/home".to_string())
}

fn alpine_root() -> String {
    std::env::var("ALPINE_ROOT").unwrap_or_else(|_| format!("{}/alpine", termux_home()))
}

fn proot_bin() -> String {
    std::env::var("PROOT_BIN").unwrap_or_else(|_| {
        let prefix = termux_prefix();
        format!("{}/bin/proot", prefix)
    })
}

/// Check if we are running inside Termux/Android
fn is_android() -> bool {
    std::path::Path::new("/data/data/com.termux").exists() || std::env::var("ANDROID_DATA").is_ok()
}

/// Check if Alpine PRoot is set up
fn alpine_available() -> bool {
    let root = alpine_root();
    std::path::Path::new(&format!("{}/usr/bin/apk", root)).exists()
        || std::path::Path::new(&format!("{}/sbin/apk", root)).exists()
        || std::path::Path::new("/usr/bin/apk").exists()
        || std::path::Path::new("/sbin/apk").exists()
}

// ---------------------------------------------------------------------------
// Run a command inside Alpine PRoot
// ---------------------------------------------------------------------------

fn proot_run(args: &[&str]) -> std::io::Result<std::process::Output> {
    if std::path::Path::new("/usr/bin/apk").exists() || std::path::Path::new("/sbin/apk").exists() {
        if args.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "empty args",
            ));
        }
        return Command::new(args[0]).args(&args[1..]).output();
    }
    let root = alpine_root();
    let proot_bin = proot_bin();
    let host_home = termux_home();

    let mut cmd = Command::new(&proot_bin);
    cmd.arg("--rootfs")
        .arg(&root)
        .arg("--bind=/dev")
        .arg("--bind=/proc")
        .arg("--bind=/sys")
        .arg(format!("--bind={}", host_home))
        .arg("-0")
        .arg("/usr/bin/env")
        .arg("PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
        .arg("HOME=/root")
        .arg("TERM=xterm-256color");

    for a in args {
        cmd.arg(a);
    }

    cmd.output()
}

/// Run in Termux environment directly
fn termux_run(cmd_name: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
    let prefix = termux_prefix();
    let bin = format!("{}/bin/{}", prefix, cmd_name);
    let exe = if std::path::Path::new(&bin).exists() {
        bin
    } else {
        cmd_name.to_string()
    };
    Command::new(exe).args(args).output()
}

// ---------------------------------------------------------------------------
// Language spec: what to install and how to verify
// ---------------------------------------------------------------------------

struct LangSpec {
    /// Termux packages (via pkg/apk install directly)
    termux_pkgs: &'static [&'static str],
    /// Alpine apk packages inside PRoot
    alpine_pkgs: &'static [&'static str],
    /// Extra post-install commands to run in Alpine PRoot
    post_install: &'static [&'static [&'static str]],
    /// Binary to check for "is installed"
    check_bin: &'static str,
}

fn get_spec(lang: &str) -> Option<LangSpec> {
    match lang {
        "rust" => Some(LangSpec {
            termux_pkgs: &["rust"],
            alpine_pkgs: &["rust", "cargo"],
            post_install: &[&["cargo", "install", "rust-analyzer"]],
            check_bin: "rustc",
        }),
        "go" => Some(LangSpec {
            termux_pkgs: &["golang"],
            alpine_pkgs: &["go"],
            post_install: &[&["go", "install", "golang.org/x/tools/gopls@latest"]],
            check_bin: "go",
        }),
        "python" => Some(LangSpec {
            termux_pkgs: &["python", "python-pip"],
            alpine_pkgs: &["python3", "py3-pip"],
            post_install: &[&["pip3", "install", "python-lsp-server[all]", "pyright"]],
            check_bin: "python3",
        }),
        "javascript" | "typescript" => Some(LangSpec {
            termux_pkgs: &["nodejs-lts"],
            alpine_pkgs: &["nodejs", "npm"],
            post_install: &[
                &[
                    "npm",
                    "install",
                    "-g",
                    "typescript-language-server",
                    "typescript",
                ],
                &["npm", "install", "-g", "vscode-langservers-extracted"],
            ],
            check_bin: "node",
        }),
        "c" | "cpp" => Some(LangSpec {
            termux_pkgs: &["clang", "build-essential"],
            alpine_pkgs: &["gcc", "g++", "musl-dev", "clang", "clang-dev"],
            post_install: &[],
            check_bin: "gcc",
        }),
        "java" => Some(LangSpec {
            termux_pkgs: &["openjdk-17"],
            alpine_pkgs: &["openjdk17"],
            post_install: &[],
            check_bin: "java",
        }),
        "kotlin" => Some(LangSpec {
            termux_pkgs: &["kotlin"],
            alpine_pkgs: &["kotlin"],
            post_install: &[],
            check_bin: "kotlin",
        }),
        "ruby" => Some(LangSpec {
            termux_pkgs: &["ruby"],
            alpine_pkgs: &["ruby", "ruby-dev"],
            post_install: &[&["gem", "install", "solargraph"]],
            check_bin: "ruby",
        }),
        "dart" => Some(LangSpec {
            termux_pkgs: &["dart"],
            alpine_pkgs: &["dart"],
            post_install: &[],
            check_bin: "dart",
        }),
        "php" => Some(LangSpec {
            termux_pkgs: &["php"],
            alpine_pkgs: &["php", "php-dev"],
            post_install: &[&["composer", "global", "require", "phpactor/phpactor"]],
            check_bin: "php",
        }),
        "haskell" => Some(LangSpec {
            termux_pkgs: &["ghc", "cabal-install"],
            alpine_pkgs: &["ghc", "cabal"],
            post_install: &[
                &["cabal", "update"],
                &["cabal", "install", "haskell-language-server"],
            ],
            check_bin: "ghc",
        }),
        "scala" => Some(LangSpec {
            termux_pkgs: &["scala"],
            alpine_pkgs: &["scala"],
            post_install: &[],
            check_bin: "scala",
        }),
        "elixir" => Some(LangSpec {
            termux_pkgs: &["elixir"],
            alpine_pkgs: &["elixir"],
            post_install: &[],
            check_bin: "elixir",
        }),
        "lua" => Some(LangSpec {
            termux_pkgs: &["lua54"],
            alpine_pkgs: &["lua5.4"],
            post_install: &[],
            check_bin: "lua",
        }),
        "perl" => Some(LangSpec {
            termux_pkgs: &["perl"],
            alpine_pkgs: &["perl"],
            post_install: &[],
            check_bin: "perl",
        }),
        "r" => Some(LangSpec {
            termux_pkgs: &["r-base"],
            alpine_pkgs: &["R"],
            post_install: &[&[
                "Rscript",
                "-e",
                "install.packages('languageserver', repos='https://cran.r-project.org')",
            ]],
            check_bin: "Rscript",
        }),
        "julia" => Some(LangSpec {
            termux_pkgs: &["julia"],
            alpine_pkgs: &["julia"],
            post_install: &[],
            check_bin: "julia",
        }),
        "swift" => Some(LangSpec {
            termux_pkgs: &["swift"],
            alpine_pkgs: &[],
            post_install: &[],
            check_bin: "swift",
        }),
        "bash" => Some(LangSpec {
            termux_pkgs: &["bash"],
            alpine_pkgs: &["bash"],
            post_install: &[&["npm", "install", "-g", "bash-language-server"]],
            check_bin: "bash",
        }),
        "nim" => Some(LangSpec {
            termux_pkgs: &["nim"],
            alpine_pkgs: &["nim"],
            post_install: &[],
            check_bin: "nim",
        }),
        "zig" => Some(LangSpec {
            termux_pkgs: &["zig"],
            alpine_pkgs: &["zig"],
            post_install: &[],
            check_bin: "zig",
        }),
        "sql" => Some(LangSpec {
            termux_pkgs: &["sqlite"],
            alpine_pkgs: &["sqlite"],
            post_install: &[&["go", "install", "github.com/lighttiger2505/sqls@latest"]],
            check_bin: "sqlite3",
        }),
        "csharp" => Some(LangSpec {
            termux_pkgs: &["dotnet-sdk-8"],
            alpine_pkgs: &[],
            post_install: &[],
            check_bin: "dotnet",
        }),
        "d" => Some(LangSpec {
            termux_pkgs: &["ldc"],
            alpine_pkgs: &["ldc"],
            post_install: &[],
            check_bin: "ldc2",
        }),
        "ocaml" => Some(LangSpec {
            termux_pkgs: &["ocaml", "opam"],
            alpine_pkgs: &["ocaml", "opam"],
            post_install: &[
                &["opam", "init", "--disable-sandboxing", "-y"],
                &["opam", "install", "ocaml-lsp-server", "-y"],
            ],
            check_bin: "ocaml",
        }),
        "clojure" => Some(LangSpec {
            termux_pkgs: &["clojure"],
            alpine_pkgs: &["clojure"],
            post_install: &[],
            check_bin: "clojure",
        }),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn install_language_handler(
    Json(payload): Json<InstallLanguageRequest>,
) -> Json<InstallLanguageResponse> {
    let lang = payload.language.to_lowercase();

    let spec = match get_spec(&lang) {
        Some(s) => s,
        None => {
            return Json(InstallLanguageResponse {
                success: false,
                message: format!("Unknown language: {}", lang),
                output: String::new(),
            })
        }
    };

    let mut all_output = String::new();

    // ---- Strategy 1: Alpine PRoot (preferred if available) ----
    if is_android() && alpine_available() {
        // Install via apk inside PRoot
        if !spec.alpine_pkgs.is_empty() {
            let mut apk_args = vec!["apk", "add", "--no-cache"];
            apk_args.extend_from_slice(spec.alpine_pkgs);
            match proot_run(&apk_args) {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    all_output.push_str(&stdout);
                    all_output.push_str(&stderr);
                    if !out.status.success() {
                        return Json(InstallLanguageResponse {
                            success: false,
                            message: format!("apk add failed for {}", lang),
                            output: all_output,
                        });
                    }
                }
                Err(e) => {
                    // Fall through to Termux strategy
                    all_output.push_str(&format!(
                        "PRoot apk failed: {}, falling back to Termux\n",
                        e
                    ));
                }
            }
        }

        // Post-install commands inside PRoot
        for cmd_args in spec.post_install {
            match proot_run(cmd_args) {
                Ok(out) => {
                    all_output.push_str(&String::from_utf8_lossy(&out.stdout));
                    all_output.push_str(&String::from_utf8_lossy(&out.stderr));
                }
                Err(e) => {
                    all_output.push_str(&format!("Post-install warn: {}\n", e));
                }
            }
        }

        return Json(InstallLanguageResponse {
            success: true,
            message: format!("{} installed via Alpine PRoot", lang),
            output: all_output,
        });
    }

    // ---- Strategy 2: Termux pkg directly ----
    if is_android() && !spec.termux_pkgs.is_empty() {
        let prefix = termux_prefix();
        let pkg_bin = format!("{}/bin/pkg", prefix);
        let mut pkg_args: Vec<&str> = vec!["install", "-y"];
        pkg_args.extend_from_slice(spec.termux_pkgs);

        match Command::new(&pkg_bin).args(&pkg_args).output() {
            Ok(out) => {
                all_output.push_str(&String::from_utf8_lossy(&out.stdout));
                all_output.push_str(&String::from_utf8_lossy(&out.stderr));
                if !out.status.success() {
                    return Json(InstallLanguageResponse {
                        success: false,
                        message: format!("pkg install failed for {}", lang),
                        output: all_output,
                    });
                }
            }
            Err(e) => {
                return Json(InstallLanguageResponse {
                    success: false,
                    message: format!("pkg not available: {}", e),
                    output: all_output,
                });
            }
        }

        // Post-install
        for cmd_slice in spec.post_install {
            if let Some((cmd, args)) = cmd_slice.split_first() {
                let _ = termux_run(cmd, args);
            }
        }

        return Json(InstallLanguageResponse {
            success: true,
            message: format!("{} installed via Termux pkg", lang),
            output: all_output,
        });
    }

    // ---- Strategy 3: Linux (apt/pacman etc.) for desktop testing ----
    #[cfg(target_os = "linux")]
    {
        if !spec.termux_pkgs.is_empty() {
            let pkg = spec.termux_pkgs[0];
            let out = Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "apt-get install -y {} || pacman -S --noconfirm {}",
                    pkg, pkg
                ))
                .output();
            if let Ok(o) = out {
                all_output.push_str(&String::from_utf8_lossy(&o.stdout));
            }
        }
        return Json(InstallLanguageResponse {
            success: true,
            message: format!("{} install attempted on Linux desktop", lang),
            output: all_output,
        });
    }

    Json(InstallLanguageResponse {
        success: false,
        message: "Platform not supported for auto-install. Please install manually via Termux."
            .to_string(),
        output: all_output,
    })
}

pub async fn check_language_handler(
    Json(payload): Json<CheckLanguageRequest>,
) -> Json<CheckLanguageResponse> {
    let lang = payload.language.to_lowercase();
    let spec = match get_spec(&lang) {
        Some(s) => s,
        None => {
            return Json(CheckLanguageResponse {
                installed: false,
                language: lang,
            })
        }
    };

    // Check in Alpine PRoot first
    if is_android() && alpine_available() {
        let check = proot_run(&["which", spec.check_bin]);
        if let Ok(out) = check {
            if out.status.success() {
                return Json(CheckLanguageResponse {
                    installed: true,
                    language: lang,
                });
            }
        }
    }

    // Check in Termux PATH
    let prefix = termux_prefix();
    let bin_path = format!("{}/bin/{}", prefix, spec.check_bin);
    if std::path::Path::new(&bin_path).exists() {
        return Json(CheckLanguageResponse {
            installed: true,
            language: lang,
        });
    }

    // Check system PATH
    if let Ok(out) = Command::new("which").arg(spec.check_bin).output() {
        if out.status.success() {
            return Json(CheckLanguageResponse {
                installed: true,
                language: lang,
            });
        }
    }

    Json(CheckLanguageResponse {
        installed: false,
        language: lang,
    })
}
