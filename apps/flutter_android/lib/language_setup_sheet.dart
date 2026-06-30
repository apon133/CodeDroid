import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:path_provider/path_provider.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'linux_manager.dart';

class LanguageSetupSheet extends StatefulWidget {
  const LanguageSetupSheet({super.key});

  @override
  State<LanguageSetupSheet> createState() => _LanguageSetupSheetState();
}

class _LanguageSetupSheetState extends State<LanguageSetupSheet> with SingleTickerProviderStateMixin {
  late TabController _tabController;
  final TextEditingController _gitUrlController = TextEditingController();
  
  String _gitCloneStatus = "Idle";
  String _gitCloneLogs = "";

  final List<Map<String, dynamic>> _languages = [
    {
      "name": "Rust",
      "package": "rust cargo",
      "binary": "usr/bin/rustc",
      "description": "Rust compiler, Cargo package manager, and rust-analyzer",
      "svg": "assets/www/assets/icons/rust.svg",
      "icon": Icons.memory,
      "category": "language",
    },
    {
      "name": "Go",
      "package": "go",
      "binary": "usr/bin/go",
      "description": "Go compiler, modules tool, and gopls LSP",
      "svg": "assets/www/assets/icons/go.svg",
      "icon": Icons.align_vertical_bottom,
      "category": "language",
    },
    {
      "name": "Dart",
      "package": "dart",
      "binary": "usr/bin/dart",
      "description": "Dart SDK with built-in Analysis Server LSP",
      "svg": "assets/www/assets/icons/dart.svg",
      "icon": Icons.flutter_dash,
      "category": "language",
    },
    {
      "name": "C",
      "package": "gcc",
      "binary": "usr/bin/gcc",
      "description": "GCC compiler and clangd LSP for C projects",
      "svg": "assets/www/assets/icons/c.svg",
      "icon": Icons.settings_ethernet,
      "category": "language",
    },
    {
      "name": "C++",
      "package": "g++",
      "binary": "usr/bin/g++",
      "description": "G++ compiler and clangd LSP for C++ projects",
      "svg": "assets/www/assets/icons/cpp.svg",
      "icon": Icons.settings_ethernet,
      "category": "language",
    },
    {
      "name": "C#",
      "package": "csharp",
      "binary": "usr/bin/dotnet",
      "description": ".NET SDK for building C# CLI applications",
      "svg": "assets/www/assets/icons/csharp.svg",
      "icon": Icons.code,
      "category": "language",
    },
    {
      "name": "Java",
      "package": "openjdk17",
      "binary": "usr/bin/javac",
      "description": "OpenJDK 17 compiler and eclipse-jdt-ls LSP",
      "svg": "assets/www/assets/icons/java.svg",
      "icon": Icons.coffee,
      "category": "language",
    },
    {
      "name": "Python",
      "package": "python3",
      "binary": "usr/bin/python3",
      "description": "Python interpreter, pip, and python-lsp-server",
      "svg": "assets/www/assets/icons/python.svg",
      "icon": Icons.code,
      "category": "language",
    },
    {
      "name": "Kotlin",
      "package": "kotlin",
      "binary": "usr/bin/kotlinc",
      "description": "Kotlin compiler for JVM and kotlin-language-server",
      "svg": "assets/www/assets/icons/kotlin.svg",
      "icon": Icons.android,
      "category": "language",
    },
    {
      "name": "Swift",
      "package": "swift",
      "binary": "usr/bin/swift",
      "description": "Swift compiler and sourcekit-lsp LSP",
      "svg": "assets/www/assets/icons/swift.svg",
      "icon": Icons.keyboard_arrow_right,
      "category": "language",
    },
    {
      "name": "Ruby",
      "package": "ruby",
      "binary": "usr/bin/ruby",
      "description": "Ruby interpreter and solargraph LSP gem",
      "svg": "assets/www/assets/icons/ruby.svg",
      "icon": Icons.diamond,
      "category": "language",
    },
    {
      "name": "JavaScript",
      "package": "javascript",
      "binary": "usr/bin/node",
      "description": "Node.js JavaScript runtime, npm, and typescript-language-server",
      "svg": "assets/www/assets/icons/javascript.svg",
      "icon": Icons.javascript,
      "category": "language",
    },
    {
      "name": "TypeScript",
      "package": "typescript",
      "binary": "usr/bin/tsc",
      "description": "TypeScript compiler and typescript-language-server",
      "svg": "assets/www/assets/icons/typescript.svg",
      "icon": Icons.code,
      "category": "language",
    },

    // ---------------- Web Frameworks ----------------
    {
      "name": "Vanilla JS",
      "package": "vanilla-js",
      "binary": "usr/bin/node",
      "description": "Pure JavaScript templates and standard Web APIs",
      "svg": "assets/www/assets/icons/javascript.svg",
      "icon": Icons.web,
      "category": "framework",
    },
    {
      "name": "React",
      "package": "react",
      "binary": "usr/bin/node",
      "description": "React.js template powered by Vite with tailwind-lsp support",
      "svg": "assets/www/assets/icons/react.svg",
      "icon": Icons.web,
      "category": "framework",
    },
    {
      "name": "Vue",
      "package": "vue",
      "binary": "usr/bin/node",
      "description": "Vue.js 3 template powered by Vite and volar language server",
      "svg": "assets/www/assets/icons/vue.svg",
      "icon": Icons.web_stories,
      "category": "framework",
    },
    {
      "name": "Svelte",
      "package": "svelte",
      "binary": "usr/bin/node",
      "description": "Svelte template powered by Vite and svelte-language-server",
      "svg": "assets/www/assets/icons/svelte.svg",
      "icon": Icons.space_dashboard,
      "category": "framework",
    },
    {
      "name": "Angular",
      "package": "angular",
      "binary": "usr/bin/node",
      "description": "Angular template powered by Angular CLI and angular-language-server",
      "svg": "assets/www/assets/icons/angular.svg",
      "icon": Icons.view_headline,
      "category": "framework",
    },
    {
      "name": "Next.js",
      "package": "nextjs",
      "binary": "usr/bin/node",
      "description": "Full-stack React framework featuring Next.js App Router support",
      "svg": "assets/www/assets/icons/nextjs.svg",
      "icon": Icons.web,
      "category": "framework",
    },
    {
      "name": "Remix",
      "package": "remix",
      "binary": "usr/bin/node",
      "description": "Full-stack Remix framework utilizing Vite and Web Standards",
      "svg": "assets/www/assets/icons/generic.svg",
      "icon": Icons.web,
      "category": "framework",
    },

    // ---------------- System Tools ----------------
    {
      "name": "Git Version Control",
      "package": "git",
      "binary": "usr/bin/git",
      "description": "Distributed version control system for tracking changes",
      "svg": "assets/www/assets/icons/git.svg",
      "icon": Icons.merge_type,
      "category": "tool",
    },
  ];

  final Map<String, String> _statuses = {};
  final Map<String, String> _logs = {};
  bool _isLoadingStatuses = true;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 3, vsync: this);
    _checkAllStatuses();
  }

  @override
  void dispose() {
    _tabController.dispose();
    _gitUrlController.dispose();
    super.dispose();
  }

  Future<void> _checkAllStatuses() async {
    if (!mounted) return;
    setState(() => _isLoadingStatuses = true);
    final appDir = await getApplicationSupportDirectory();
    final rootfsPath = "${appDir.path}/linux/rootfs";

    for (var lang in _languages) {
      final binaryPath = "$rootfsPath/${lang['binary']}";
      final binaryFile = File(binaryPath);
      final binaryLink = Link(binaryPath);
      if (binaryFile.existsSync() || binaryLink.existsSync()) {
        _statuses[lang['package']] = "Installed";
      } else {
        _statuses[lang['package']] = "Not Installed";
      }
    }

    if (!mounted) return;
    setState(() => _isLoadingStatuses = false);
  }

  Future<void> _installLanguage(String package) async {
    if (!mounted) return;
    setState(() {
      _statuses[package] = "Installing";
      _logs[package] = "Initializing installation...\n";
    });

    await LinuxManager.runApkAdd(package, (log) {
      if (mounted) {
        setState(() {
          _logs[package] = (_logs[package] ?? "") + log;
        });
      }
    });

    if (!mounted) return;
    await _checkAllStatuses();
  }

  Future<void> _deleteLanguage(String package) async {
    if (!mounted) return;
    setState(() {
      _statuses[package] = "Deleting";
      _logs[package] = "Initializing removal...\n";
    });

    await LinuxManager.deletePackage(package, (log) {
      if (mounted) {
        setState(() {
          _logs[package] = (_logs[package] ?? "") + log;
        });
      }
    });

    if (!mounted) return;
    await _checkAllStatuses();
  }

  Future<void> _cloneGitRepo(String repoUrl) async {
    if (repoUrl.trim().isEmpty) return;
    setState(() {
      _gitCloneStatus = "Cloning";
      _gitCloneLogs = "Initializing Git clone for $repoUrl...\n";
    });

    // Extract repo name
    String repoName = repoUrl.split('/').last.replaceAll('.git', '').trim();
    if (repoName.isEmpty) repoName = "repo";
    
    final targetPath = "/root/$repoName";

    try {
      await LinuxManager.runGuestCommand([
        "/usr/bin/git",
        "clone",
        repoUrl.trim(),
        targetPath,
      ], (log) {
        if (mounted) {
          setState(() {
            _gitCloneLogs += log;
          });
        }
      });
      if (mounted) {
        setState(() {
          _gitCloneStatus = "Success";
          _gitCloneLogs += "\nSUCCESS: Repository cloned to $targetPath successfully!";
        });
        _gitUrlController.clear();
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _gitCloneStatus = "Failed";
          _gitCloneLogs += "\nERROR: Git clone failed: $e";
        });
      }
    }
  }

  Widget _buildLanguageIcon(Map<String, dynamic> lang, BuildContext context) {
    final String? svgPath = lang['svg'];
    if (svgPath != null) {
      return Container(
        padding: const EdgeInsets.all(6),
        decoration: BoxDecoration(
          color: const Color(0xFF262626),
          borderRadius: BorderRadius.circular(10),
          border: Border.all(color: const Color(0xFF383838), width: 1),
        ),
        width: 42,
        height: 42,
        child: SvgPicture.asset(
          svgPath,
          fit: BoxFit.contain,
          placeholderBuilder: (context) => Icon(
            lang['icon'] as IconData? ?? Icons.code,
            color: Theme.of(context).colorScheme.primary,
            size: 22,
          ),
        ),
      );
    }
    return Container(
      padding: const EdgeInsets.all(6),
      decoration: BoxDecoration(
        color: const Color(0xFF262626),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: const Color(0xFF383838), width: 1),
      ),
      width: 42,
      height: 42,
      child: Icon(
        lang['icon'] as IconData? ?? Icons.code,
        color: Theme.of(context).colorScheme.primary,
        size: 22,
      ),
    );
  }

  Widget _buildPackageList(String category) {
    final filtered = _languages.where((l) => l['category'] == category).toList();

    return ListView.builder(
      physics: const BouncingScrollPhysics(),
      itemCount: filtered.length,
      itemBuilder: (context, index) {
        final lang = filtered[index];
        final package = lang['package'];
        final status = _statuses[package] ?? "Not Installed";
        final isInstalling = status == "Installing" || status == "Deleting";
        final isInstalled = status == "Installed";

        return Container(
          margin: const EdgeInsets.symmetric(vertical: 6),
          decoration: BoxDecoration(
            color: const Color(0xFF1D1D1D),
            borderRadius: BorderRadius.circular(14),
            border: Border.all(
              color: status == "Deleting"
                  ? Colors.redAccent.withOpacity(0.4)
                  : isInstalling
                      ? const Color(0xFF228DF2).withOpacity(0.4)
                      : isInstalled
                          ? const Color(0xFF15AC91).withOpacity(0.2)
                          : const Color(0xFF2E2E2E),
              width: 1,
            ),
          ),
          child: ExpansionTile(
            iconColor: Colors.white,
            collapsedIconColor: Colors.white60,
            shape: const Border(),
            leading: _buildLanguageIcon(lang, context),
            title: Text(
              lang['name'],
              style: const TextStyle(
                color: Colors.white,
                fontWeight: FontWeight.w600,
                fontSize: 15,
              ),
            ),
            subtitle: Row(
              children: [
                Container(
                  width: 6,
                  height: 6,
                  decoration: BoxDecoration(
                    color: status == "Deleting"
                        ? Colors.redAccent
                        : isInstalling
                            ? const Color(0xFF228DF2)
                            : isInstalled
                                ? const Color(0xFF15AC91)
                                : Colors.grey,
                    shape: BoxShape.circle,
                  ),
                ),
                const SizedBox(width: 6),
                Text(
                  status == "Deleting"
                      ? "Removing LSP..."
                      : isInstalling
                          ? "Installing LSP..."
                          : status,
                  style: TextStyle(
                    color: status == "Deleting"
                        ? Colors.redAccent
                        : isInstalling
                            ? const Color(0xFF228DF2)
                            : isInstalled
                                ? const Color(0xFF15AC91)
                                : Colors.grey,
                    fontSize: 12,
                  ),
                ),
              ],
            ),
            trailing: isInstalling
                ? const SizedBox(
                    width: 22,
                    height: 22,
                    child: CircularProgressIndicator(
                      strokeWidth: 2,
                      valueColor: AlwaysStoppedAnimation<Color>(Color(0xFF228DF2)),
                    ),
                  )
                : isInstalled
                    ? ElevatedButton(
                        onPressed: () => _deleteLanguage(package),
                        style: ElevatedButton.styleFrom(
                          backgroundColor: const Color(0xFFD32F2F).withOpacity(0.15),
                          foregroundColor: Colors.redAccent,
                          elevation: 0,
                          side: BorderSide(color: Colors.redAccent.withOpacity(0.3), width: 1),
                          shape: RoundedRectangleBorder(
                            borderRadius: BorderRadius.circular(20),
                          ),
                          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 0),
                          minimumSize: const Size(80, 32),
                        ),
                        child: const Text(
                          "Delete",
                          style: TextStyle(
                            fontWeight: FontWeight.bold,
                            fontSize: 12,
                            color: Colors.redAccent,
                          ),
                        ),
                      )
                    : ElevatedButton(
                        onPressed: () => _installLanguage(package),
                        style: ElevatedButton.styleFrom(
                          backgroundColor: const Color(0xFF228DF2),
                          foregroundColor: Colors.black,
                          elevation: 0,
                          shape: RoundedRectangleBorder(
                            borderRadius: BorderRadius.circular(20),
                          ),
                          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 0),
                          minimumSize: const Size(80, 32),
                        ),
                        child: const Text(
                          "Install",
                          style: TextStyle(
                            fontWeight: FontWeight.bold,
                            fontSize: 12,
                            color: Colors.black,
                          ),
                        ),
                      ),
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 4, 16, 16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Divider(color: Color(0xFF2A2A2A), height: 16),
                    Text(
                      lang['description'],
                      style: const TextStyle(color: Colors.white70, fontSize: 13, height: 1.4),
                    ),
                    if (package == "git" && isInstalled) ...[
                      const SizedBox(height: 16),
                      const Text(
                        "Clone External Git Repository",
                        style: TextStyle(
                          color: Colors.white,
                          fontSize: 13,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      const SizedBox(height: 8),
                      Row(
                        children: [
                          Expanded(
                            child: TextField(
                              controller: _gitUrlController,
                              style: const TextStyle(color: Colors.white, fontSize: 13, fontFamily: 'monospace'),
                              decoration: InputDecoration(
                                hintText: "https://github.com/user/repo.git",
                                hintStyle: TextStyle(color: Colors.grey[600], fontSize: 13),
                                contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                                filled: true,
                                fillColor: const Color(0xFF161616),
                                enabledBorder: OutlineInputBorder(
                                  borderRadius: BorderRadius.circular(8),
                                  borderSide: const BorderSide(color: Color(0xFF2A2A2A)),
                                ),
                                focusedBorder: OutlineInputBorder(
                                  borderRadius: BorderRadius.circular(8),
                                  borderSide: const BorderSide(color: Color(0xFF228DF2)),
                                ),
                              ),
                            ),
                          ),
                          const SizedBox(width: 8),
                          ElevatedButton.icon(
                            onPressed: _gitCloneStatus == "Cloning"
                                ? null
                                : () => _cloneGitRepo(_gitUrlController.text),
                            icon: const Icon(Icons.download, size: 14),
                            label: const Text("Clone"),
                            style: ElevatedButton.styleFrom(
                              backgroundColor: const Color(0xFF15AC91),
                              foregroundColor: Colors.black,
                              padding: const EdgeInsets.symmetric(horizontal: 14),
                              shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                            ),
                          ),
                        ],
                      ),
                      if (_gitCloneLogs.isNotEmpty) ...[
                        const SizedBox(height: 12),
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            Text(
                              "Git Clone Output ($_gitCloneStatus):",
                              style: const TextStyle(color: Colors.grey, fontSize: 11, fontWeight: FontWeight.bold),
                            ),
                            TextButton(
                              onPressed: () {
                                setState(() {
                                  _gitCloneLogs = "";
                                  _gitCloneStatus = "Idle";
                                });
                              },
                              child: const Text("Clear Logs", style: TextStyle(fontSize: 11, color: Colors.redAccent)),
                            ),
                          ],
                        ),
                        Container(
                          width: double.infinity,
                          height: 120,
                          padding: const EdgeInsets.all(8),
                          decoration: BoxDecoration(
                            color: Colors.black,
                            borderRadius: BorderRadius.circular(8),
                            border: Border.all(color: const Color(0xFF2A2A2A)),
                          ),
                          child: SingleChildScrollView(
                            reverse: true,
                            child: SelectableText(
                              _gitCloneLogs,
                              style: const TextStyle(
                                color: Color(0xFF00FF00),
                                fontFamily: 'monospace',
                                fontSize: 11,
                              ),
                            ),
                          ),
                        ),
                      ],
                    ],
                    if (_logs[package] != null) ...[
                      const SizedBox(height: 16),
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          const Text(
                            "Setup & LSP Installation Log:",
                            style: TextStyle(color: Colors.grey, fontSize: 11, fontWeight: FontWeight.bold),
                          ),
                          TextButton.icon(
                            onPressed: () {
                              Clipboard.setData(ClipboardData(text: _logs[package]!));
                              ScaffoldMessenger.of(context).showSnackBar(
                                const SnackBar(
                                  content: Text("Installation log copied to clipboard"),
                                  duration: Duration(seconds: 2),
                                ),
                              );
                            },
                            icon: const Icon(Icons.copy, size: 12, color: Color(0xFF228DF2)),
                            label: const Text(
                              "Copy Logs",
                              style: TextStyle(fontSize: 11, color: Color(0xFF228DF2)),
                            ),
                            style: TextButton.styleFrom(
                              padding: EdgeInsets.zero,
                              minimumSize: const Size(50, 30),
                              tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 6),
                      Container(
                        width: double.infinity,
                        height: 140,
                        padding: const EdgeInsets.all(8),
                        decoration: BoxDecoration(
                          color: Colors.black,
                          borderRadius: BorderRadius.circular(8),
                          border: Border.all(color: const Color(0xFF2A2A2A)),
                        ),
                        child: SingleChildScrollView(
                          reverse: true,
                          child: SelectableText(
                            _logs[package]!,
                            style: const TextStyle(
                              color: Color(0xFF00FF00),
                              fontFamily: 'monospace',
                              fontSize: 11,
                            ),
                          ),
                        ),
                      ),
                    ],
                  ],
                ),
              )
            ],
          ),
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: const BoxDecoration(
        color: Color(0xFF161616),
        borderRadius: BorderRadius.vertical(top: Radius.circular(24)),
      ),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
      height: MediaQuery.of(context).size.height * 0.85,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Drag Handle
          Center(
            child: Container(
              width: 36,
              height: 4,
              decoration: BoxDecoration(
                color: Colors.grey[800],
                borderRadius: BorderRadius.circular(10),
              ),
            ),
          ),
          const SizedBox(height: 16),

          // Title & Header
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                "Development Setup Hub",
                style: Theme.of(context).textTheme.titleLarge?.copyWith(
                      color: Colors.white,
                      fontWeight: FontWeight.bold,
                    ),
              ),
              IconButton(
                icon: const Icon(Icons.close, color: Colors.white60, size: 20),
                onPressed: () => Navigator.pop(context),
              )
            ],
          ),
          const SizedBox(height: 4),
          const Text(
            "Configure compilers, runtimes, LSP services, and source control repositories with one-click automation.",
            style: TextStyle(
              color: Colors.grey,
              fontSize: 13,
              height: 1.3,
            ),
          ),
          const SizedBox(height: 16),

          // Custom TabBar
          Container(
            height: 38,
            decoration: BoxDecoration(
              color: const Color(0xFF1D1D1D),
              borderRadius: BorderRadius.circular(10),
            ),
            child: TabBar(
              controller: _tabController,
              indicator: BoxDecoration(
                borderRadius: BorderRadius.circular(8),
                color: const Color(0xFF2D2D2D),
              ),
              indicatorSize: TabBarIndicatorSize.tab,
              dividerColor: Colors.transparent,
              labelColor: Colors.white,
              unselectedLabelColor: Colors.grey,
              labelStyle: const TextStyle(fontWeight: FontWeight.bold, fontSize: 13),
              tabs: const [
                Tab(text: "Languages"),
                Tab(text: "Frameworks"),
                Tab(text: "System Tools"),
              ],
            ),
          ),
          const SizedBox(height: 12),

          // Tab Bar Views
          Expanded(
            child: _isLoadingStatuses
                ? const Center(
                    child: CircularProgressIndicator(
                      valueColor: AlwaysStoppedAnimation<Color>(Color(0xFF228DF2)),
                    ),
                  )
                : TabBarView(
                    controller: _tabController,
                    children: [
                      _buildPackageList("language"),
                      _buildPackageList("framework"),
                      _buildPackageList("tool"),
                    ],
                  ),
          ),
        ],
      ),
    );
  }
}
