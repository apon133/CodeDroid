import 'dart:convert';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';
import 'language_setup_sheet.dart';
import 'linux_manager.dart';

class InteractiveTerminalScreen extends StatefulWidget {
  const InteractiveTerminalScreen({super.key});

  @override
  State<InteractiveTerminalScreen> createState() => _InteractiveTerminalScreenState();
}

class _InteractiveTerminalScreenState extends State<InteractiveTerminalScreen> {
  Process? _process;
  final List<String> _output = [];
  final TextEditingController _inputController = TextEditingController();
  final ScrollController _scrollController = ScrollController();
  final FocusNode _focusNode = FocusNode();
  bool _isRunning = false;

  final List<String> _history = [];
  int _historyIndex = -1;

  @override
  void initState() {
    super.initState();
    _startSession();
  }

  @override
  void dispose() {
    _process?.kill();
    _inputController.dispose();
    _scrollController.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  Future<void> _startSession() async {
    if (_process != null) {
      _process!.kill();
      _process = null;
    }

    setState(() {
      _output.clear();
      _output.add("Connecting to CodeDroid Alpine Linux environment...\n");
      _isRunning = true;
      _historyIndex = -1;
    });

    try {
      final appDir = await getApplicationSupportDirectory();
      final String appDirCanonical = LinuxManager.canonicalizePath(appDir.path);
      final String linuxDir = "$appDirCanonical/linux";
      final String prootPath = LinuxManager.canonicalizePath("$linuxDir/proot");
      final String rootfsPath = LinuxManager.canonicalizePath("$linuxDir/rootfs");

      final tmpDir = Directory("$linuxDir/tmp");
      if (!tmpDir.existsSync()) {
        tmpDir.createSync(recursive: true);
      }
      final String tmpPath = tmpDir.resolveSymbolicLinksSync();

      final l2sDir = Directory("$rootfsPath/.l2s");
      if (!l2sDir.existsSync()) {
        l2sDir.createSync(recursive: true);
      }
      final String l2sPath = l2sDir.resolveSymbolicLinksSync();

      // Mirror directories to .l2s to avoid ENOENT errors during link creation inside PRoot
      LinuxManager.mirrorRootfsDirectoriesToL2s(rootfsPath);

      // Ensure a guest-writable Go temp directory exists inside rootfs
      final guestRootTmp = Directory("$rootfsPath/root/tmp");
      if (!guestRootTmp.existsSync()) {
        guestRootTmp.createSync(recursive: true);
      }

      // Clear stale apk lock
      final lockFile = File("$rootfsPath/lib/apk/db/lock");
      if (lockFile.existsSync()) {
        try {
          lockFile.deleteSync();
          _appendOutput("[System: Cleared stale apk database lock]\n");
        } catch (_) {}
      }

      // Merged clean environment
      final Map<String, String> mergedEnv = LinuxManager.buildCleanEnvironment(
        tmpPath: tmpPath,
        l2sPath: l2sPath,
        appendHostPath: true,
        extraEnv: {
          "GOTMPDIR": "/root/tmp",
          "CGO_ENABLED": "0",
        },
      );

      _process = await Process.start(
        prootPath,
        [
          '-0',
          '--link2symlink',
          '-r',
          rootfsPath,
          '-w',
          '/',
          '-b',
          '/dev',
          '-b',
          '/proc',
          '-b',
          '/sys',
          '/bin/sh',
        ],
        workingDirectory: linuxDir,
        environment: mergedEnv,
      );

      _appendOutput("Session started. Welcome to guest Alpine Linux!\n\n");

      // Listen to stdout
      _process!.stdout.transform(const Utf8Decoder(allowMalformed: true)).listen((data) {
        _appendOutput(data);
      }, onError: (e) {
        _appendOutput("\n[stdout error: $e]\n");
      });

      // Listen to stderr
      _process!.stderr.transform(const Utf8Decoder(allowMalformed: true)).listen((data) {
        _appendOutput(data);
      }, onError: (e) {
        _appendOutput("\n[stderr error: $e]\n");
      });

      _process!.exitCode.then((code) {
        _appendOutput("\n[Process exited with code $code]\n");
        if (mounted) {
          setState(() {
            _isRunning = false;
            _process = null;
          });
        }
      });
    } catch (e) {
      _appendOutput("\n[Failed to start process: $e]\n");
      if (mounted) {
        setState(() {
          _isRunning = false;
        });
      }
    }
  }

  void _appendOutput(String data) {
    if (!mounted) return;
    setState(() {
      // Strip ANSI escape sequences to keep display output clean
      final cleanData = data.replaceAll(RegExp(r'\x1B\[[0-9;]*[a-zA-Z]'), '');
      _output.add(cleanData);
      
      if (_output.length > 2000) {
        _output.removeRange(0, _output.length - 2000);
      }
    });
    
    // Auto-scroll to bottom
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 30),
          curve: Curves.easeOut,
        );
      }
    });
  }

  void _sendInput(String input) {
    if (_process == null) return;
    final trimmed = input.trim();
    if (trimmed == "clear") {
      setState(() {
        _output.clear();
      });
    }
    _process!.stdin.write('$input\n');
    if (trimmed.isNotEmpty) {
      _history.add(input);
      _historyIndex = _history.length;
    }
    _inputController.clear();
    _focusNode.requestFocus();
  }

  void _sendRawKey(String sequence) {
    if (_process == null) return;
    _process!.stdin.write(sequence);
    _focusNode.requestFocus();
  }

  void _sendCtrlKey(String key) {
    if (_process == null) return;
    if (key == 'C') {
      _process!.stdin.write("\x03");
      _appendOutput("^C\n");
    } else if (key == 'D') {
      _process!.stdin.write("\x04");
      _appendOutput("^D\n");
    }
    _focusNode.requestFocus();
  }

  void _historyUp() {
    if (_history.isEmpty) return;
    if (_historyIndex > 0) {
      _historyIndex--;
      _inputController.text = _history[_historyIndex];
      _inputController.selection = TextSelection.fromPosition(
        TextPosition(offset: _inputController.text.length),
      );
    }
  }

  void _historyDown() {
    if (_history.isEmpty) return;
    if (_historyIndex < _history.length - 1) {
      _historyIndex++;
      _inputController.text = _history[_historyIndex];
    } else {
      _historyIndex = _history.length;
      _inputController.clear();
    }
    _inputController.selection = TextSelection.fromPosition(
      TextPosition(offset: _inputController.text.length),
    );
  }

  @override
  Widget build(BuildContext context) {
    const consoleStyle = TextStyle(
      fontFamily: 'monospace',
      color: Color(0xFFD0D0D0),
      fontSize: 14,
      height: 1.2,
      fontWeight: FontWeight.w400,
    );

    return Scaffold(
      backgroundColor: const Color(0xFF000000), // Termux style black
      body: SafeArea(
        child: GestureDetector(
          onTap: () => _focusNode.requestFocus(),
          behavior: HitTestBehavior.opaque,
          child: Column(
            children: [
              // Minimal status line at the top
              Container(
                color: const Color(0xFF121212),
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Row(
                      children: [
                        Container(
                          width: 6,
                          height: 6,
                          decoration: BoxDecoration(
                            color: _isRunning ? Colors.greenAccent : Colors.redAccent,
                            shape: BoxShape.circle,
                          ),
                        ),
                        const SizedBox(width: 6),
                        const Text(
                          "alpine-session",
                          style: TextStyle(
                            color: Colors.grey,
                            fontSize: 11,
                            fontFamily: 'monospace',
                          ),
                        ),
                      ],
                    ),
                    Row(
                      children: [
                        IconButton(
                          constraints: const BoxConstraints(),
                          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                          icon: const Icon(Icons.settings_suggest, color: Colors.grey, size: 16),
                          onPressed: () {
                            showModalBottomSheet(
                              context: context,
                              isScrollControlled: true,
                              backgroundColor: Colors.transparent,
                              builder: (context) => const LanguageSetupSheet(),
                            );
                          },
                        ),
                        IconButton(
                          constraints: const BoxConstraints(),
                          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                          icon: const Icon(Icons.refresh, color: Colors.grey, size: 16),
                          onPressed: _startSession,
                        ),
                        IconButton(
                          constraints: const BoxConstraints(),
                          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                          icon: const Icon(Icons.close, color: Colors.grey, size: 16),
                          onPressed: () => Navigator.pop(context),
                        ),
                      ],
                    ),
                  ],
                ),
              ),

              // Console output & inline input area
              Expanded(
                child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
                  child: SingleChildScrollView(
                    controller: _scrollController,
                    physics: const AlwaysScrollableScrollPhysics(),
                    child: SelectionArea(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          // Text history
                          Text(
                            _output.join(),
                            style: consoleStyle,
                          ),
                          
                          // Active inline input line
                          Row(
                            crossAxisAlignment: CrossAxisAlignment.center,
                            children: [
                              const Text(
                                "\$ ",
                                style: TextStyle(
                                  color: Color(0xFF00FF00), // Termux bright green
                                  fontWeight: FontWeight.bold,
                                  fontSize: 14,
                                  fontFamily: 'monospace',
                                ),
                              ),
                              Expanded(
                                child: TextField(
                                  controller: _inputController,
                                  focusNode: _focusNode,
                                  autofocus: true,
                                  style: consoleStyle,
                                  cursorColor: const Color(0xFF00FF00), // Blinking green block cursor
                                  cursorWidth: 8,
                                  cursorHeight: 16,
                                  decoration: const InputDecoration(
                                    border: InputBorder.none,
                                    isDense: true,
                                    contentPadding: EdgeInsets.zero,
                                  ),
                                  onSubmitted: _sendInput,
                                ),
                              ),
                            ],
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
              ),
              
              // Termux-style accessory buttons
              Container(
                color: const Color(0xFF1C1C1C),
                padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 4),
                child: SingleChildScrollView(
                  scrollDirection: Axis.horizontal,
                  child: Row(
                    children: [
                      _buildTermuxKey("ESC", () => _sendRawKey("\x1B")),
                      _buildTermuxKey("CTRL+C", () => _sendCtrlKey('C')),
                      _buildTermuxKey("CTRL+D", () => _sendCtrlKey('D')),
                      _buildTermuxKey("TAB", () => _sendRawKey("\t")),
                      _buildTermuxKey("─", () => _sendRawKey("-")),
                      _buildTermuxKey("/", () => _sendRawKey("/")),
                      _buildTermuxKey("▲", _historyUp),
                      _buildTermuxKey("▼", _historyDown),
                      _buildTermuxKey("◀", () => _sendRawKey("\x1b[D")),
                      _buildTermuxKey("▶", () => _sendRawKey("\x1b[C")),
                      _buildTermuxKey("apk update", () {
                        _inputController.text = "apk update";
                        _focusNode.requestFocus();
                      }),
                      _buildTermuxKey("apk add ", () {
                        _inputController.text = "apk add ";
                        _focusNode.requestFocus();
                      }),
                      _buildTermuxKey("Setup Python & Suggestions", () {
                        _inputController.text = "apk update && apk add python3 py3-pip && pip3 install python-lsp-server black --break-system-packages";
                        _focusNode.requestFocus();
                      }),

                      //  _buildTermuxKey("Setup Python & Suggestions", () {
                      //  _inputController.text = "apk update && apk add python3 py3-pip && pip3 install python-lsp-server black --break-system-packages";
                      //  _focusNode.requestFocus();
                    //  }),
                      _buildTermuxKey("clear", () => _sendInput("clear")),
                    ],
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildTermuxKey(String label, VoidCallback onPressed) {
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 3),
      child: TextButton(
        style: TextButton.styleFrom(
          backgroundColor: const Color(0xFF2C2C2C),
          foregroundColor: Colors.white,
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
          minimumSize: Size.zero,
          tapTargetSize: MaterialTapTargetSize.shrinkWrap,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(3),
          ),
        ),
        onPressed: onPressed,
        child: Text(
          label,
          style: const TextStyle(
            fontSize: 11,
            fontFamily: 'monospace',
            fontWeight: FontWeight.bold,
          ),
        ),
      ),
    );
  }
}
