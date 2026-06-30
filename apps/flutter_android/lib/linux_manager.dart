import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:path_provider/path_provider.dart';

class LinuxManager {
  static Process? _apiProcess;
  static final List<String> processLogs = [];

  static Future<void> initialize() async {
    debugPrint("CodeDroid Debug: initialize() invoked.");
    try {
      // Kill any stale background processes from a previous run
      await _killStaleProcesses();

      // 1. Get architecture of the device
      final String arch = await _getArchitecture();
      debugPrint("CodeDroid: Device architecture is $arch");

      // 2. Determine target directories
      final appDir = await getApplicationSupportDirectory();
      debugPrint("CodeDroid Debug: appDir.path='${appDir.path}'");
      final String appDirCanonical = canonicalizePath(appDir.path);
      debugPrint("CodeDroid Debug: Canonical appDir.path='$appDirCanonical'");

      final linuxDir = Directory("$appDirCanonical/linux");
      if (!linuxDir.existsSync()) {
        linuxDir.createSync(recursive: true);
      }
      final String linuxDirPath = linuxDir.resolveSymbolicLinksSync();

      final rootfsDir = Directory("$linuxDirPath/rootfs");
      final File prootFile = File("$linuxDirPath/proot");
      final File alpineTarFile = File(
        "$linuxDirPath/alpine-minirootfs.tar.gz",
      );

      debugPrint(
        "CodeDroid Debug: rootfsDir='${rootfsDir.path}', exists=${rootfsDir.existsSync()}",
      );
      debugPrint(
        "CodeDroid Debug: prootFile='${prootFile.path}', exists=${prootFile.existsSync()}",
      );

      // Check if existing proot is a valid ELF binary
      bool isProotValid = false;
      if (prootFile.existsSync() && prootFile.lengthSync() > 1000) {
        try {
          final randomAccessFile = prootFile.openSync(mode: FileMode.read);
          final bytes = randomAccessFile.readSync(4);
          randomAccessFile.closeSync();
          if (bytes.length == 4 &&
              bytes[0] == 0x7f &&
              bytes[1] == 0x45 &&
              bytes[2] == 0x4c &&
              bytes[3] == 0x46) {
            isProotValid = true;
          }
          debugPrint(
            "CodeDroid Debug: checked proot ELF magic, isProotValid=$isProotValid",
          );
        } catch (e) {
          debugPrint("CodeDroid: Error checking ELF magic: $e");
        }
      }

      // Check only for core rootfs structure (bin/sh and sbin/apk).
      final String apkPath = "${rootfsDir.path}/sbin/apk";
      final String shPath = "${rootfsDir.path}/bin/sh";
      final bool isRootfsValid =
          rootfsDir.existsSync() &&
          (File(apkPath).existsSync() || Link(apkPath).existsSync()) &&
          (File(shPath).existsSync() || Link(shPath).existsSync());
      debugPrint(
        "CodeDroid Debug: isRootfsValid=$isRootfsValid (apk exists=${File(apkPath).existsSync() || Link(apkPath).existsSync()}, sh exists=${File(shPath).existsSync() || Link(shPath).existsSync()})",
      );

      if (!isRootfsValid && linuxDir.existsSync()) {
        debugPrint(
          "CodeDroid: Rootfs core structure missing. Clearing environment for full re-extraction...",
        );
        try {
          linuxDir.deleteSync(recursive: true);
          debugPrint("CodeDroid Debug: Cleared linuxDir successfully.");
        } catch (e) {
          debugPrint("CodeDroid Debug: Failed to clear linuxDir: $e");
        }
      } else if (!isProotValid && prootFile.existsSync()) {
        debugPrint(
          "CodeDroid: PRoot binary is invalid/corrupt. Re-copying proot binary only...",
        );
        try {
          prootFile.deleteSync();
          debugPrint("CodeDroid Debug: Deleted corrupt prootFile.");
        } catch (e) {
          debugPrint("CodeDroid Debug: Failed to delete corrupt prootFile: $e");
        }
      }

      if (!linuxDir.existsSync()) {
        linuxDir.createSync(recursive: true);
        debugPrint("CodeDroid Debug: Created linuxDir.");
      }

      // 3. Extract assets if not already done or if proot/rootfs was invalid
      if (!rootfsDir.existsSync() || !prootFile.existsSync()) {
        debugPrint("CodeDroid: Initializing Linux environment...");

        // Copy proot binary from assets
        debugPrint("CodeDroid Debug: Loading assets/linux/$arch/proot...");
        final ByteData prootData = await rootBundle.load(
          "assets/linux/$arch/proot",
        );
        await prootFile.writeAsBytes(prootData.buffer.asUint8List());
        debugPrint(
          "CodeDroid Debug: Written proot binary. Setting permissions...",
        );
        final chmodProot = await Process.run("chmod", ["755", prootFile.path]);
        debugPrint(
          "CodeDroid Debug: chmod proot exitCode=${chmodProot.exitCode}, stdout='${chmodProot.stdout}', stderr='${chmodProot.stderr}'",
        );

        // Copy Alpine minirootfs from assets
        debugPrint(
          "CodeDroid Debug: Loading assets/linux/$arch/alpine-minirootfs.tar.gz...",
        );
        final ByteData alpineData = await rootBundle.load(
          "assets/linux/$arch/alpine-minirootfs.tar.gz",
        );
        await alpineTarFile.writeAsBytes(alpineData.buffer.asUint8List());
        debugPrint("CodeDroid Debug: Written alpine tarball.");

        // Extract tarball using built-in system tar
        rootfsDir.createSync(recursive: true);
        debugPrint("CodeDroid Debug: Extracting tarball...");
        final tarResult = await Process.run("tar", [
          "-xzf",
          alpineTarFile.path,
          "-C",
          rootfsDir.path,
        ]);
        debugPrint(
          "CodeDroid Debug: tar extraction exitCode=${tarResult.exitCode}, stdout='${tarResult.stdout}', stderr='${tarResult.stderr}'",
        );

        if (tarResult.exitCode != 0) {
          debugPrint(
            "CodeDroid Error: Failed to extract Alpine rootfs: ${tarResult.stderr}",
          );
          return;
        }

        // Cleanup tarball to save storage space
        await alpineTarFile.delete();
        debugPrint("CodeDroid Debug: Cleaned up tarball.");

        // Setup basic network config (DNS)
        final resolvConf = File("${rootfsDir.path}/etc/resolv.conf");
        resolvConf.createSync(recursive: true);
        resolvConf.writeAsStringSync(
          "nameserver 8.8.8.8\nnameserver 1.1.1.1\n",
        );
        debugPrint("CodeDroid Debug: resolv.conf created.");

        // Ensure guest /tmp exists
        final guestTmpDir = Directory("${rootfsDir.path}/tmp");
        if (!guestTmpDir.existsSync()) {
          guestTmpDir.createSync(recursive: true);
          debugPrint("CodeDroid Debug: guest /tmp created.");
        }
      }

      // Now resolve rootfs symbolic links
      final String rootfsPath = rootfsDir.resolveSymbolicLinksSync();
      debugPrint("CodeDroid Debug: Canonical rootfsPath='$rootfsPath'");

      // Ensure node_network_bypass.js exists inside rootfs at /usr/local/lib/node_network_bypass.js
      final File bypassFile = File(
        "$rootfsPath/usr/local/lib/node_network_bypass.js",
      );
      if (!bypassFile.parent.existsSync()) {
        bypassFile.parent.createSync(recursive: true);
      }
      bypassFile.writeAsStringSync('''
try {
  const os = require('os');
  if (os && typeof os.networkInterfaces === 'function') {
    os.networkInterfaces = () => ({
      lo: [
        {
          address: '127.0.0.1',
          netmask: '255.0.0.0',
          family: 'IPv4',
          mac: '00:00:00:00:00:00',
          internal: true,
          cidr: '127.0.0.1/8'
        }
      ]
    });
  }
} catch (e) {
  // Ignore errors
}
''');
      debugPrint("CodeDroid Debug: node_network_bypass.js written to rootfs.");

      // 4. Copy/update Codedroid API binary from assets to the rootfs
      final File apiFile = File(
        "$rootfsPath/usr/local/bin/codedroid_api",
      );
      if (rootfsDir.existsSync()) {
        try {
          debugPrint("CodeDroid Debug: Ensuring rootfs files are writable...");
          final chmodRootfs = await Process.run("chmod", [
            "-R",
            "u+rwx",
            rootfsPath,
          ]);
          debugPrint(
            "CodeDroid Debug: chmod rootfs exitCode=${chmodRootfs.exitCode}, stdout='${chmodRootfs.stdout}', stderr='${chmodRootfs.stderr}'",
          );

          // Ensure l2s directory exists and is writable on the host
          final l2sDir = Directory("$rootfsPath/.l2s");
          if (!l2sDir.existsSync()) {
            l2sDir.createSync(recursive: true);
          }
          mirrorRootfsDirectoriesToL2s(rootfsPath);
          cleanOrphanL2sMetadata(rootfsPath);
          final chmodL2s = await Process.run("chmod", [
            "-R",
            "u+rwx",
            l2sDir.path,
          ]);
          debugPrint(
            "CodeDroid Debug: chmod l2s exitCode=${chmodL2s.exitCode}, stdout='${chmodL2s.stdout}', stderr='${chmodL2s.stderr}'",
          );
        } catch (e) {
          debugPrint("CodeDroid Warning: Failed to chmod: $e");
        }

        try {
          apiFile.parent.createSync(recursive: true);
          debugPrint(
            "CodeDroid Debug: Loading assets/linux/$arch/codedroid_api...",
          );
          final ByteData apiData = await rootBundle.load(
            "assets/linux/$arch/codedroid_api",
          );
          await apiFile.writeAsBytes(apiData.buffer.asUint8List());
          final chmodApi = await Process.run("chmod", ["755", apiFile.path]);
          debugPrint(
            "CodeDroid Debug: chmod api exitCode=${chmodApi.exitCode}, stdout='${chmodApi.stdout}', stderr='${chmodApi.stderr}'",
          );
          debugPrint(
            "CodeDroid: Codedroid API binary successfully updated in rootfs.",
          );
        } catch (e) {
          debugPrint(
            "CodeDroid Info: No prebundled codedroid_api found in assets, skipping copy: $e",
          );
        }
      }

      // 5. Convert any absolute symlinks to relative to bypass PRoot / Android filesystem limitations
      _convertAbsoluteSymlinksToRelative(rootfsPath);

      // 6. Start the backend api process inside PRoot
      await _startApiServer(prootFile.path, rootfsPath);
    } catch (e, stacktrace) {
      debugPrint("CodeDroid: Error initializing Linux manager: $e");
      debugPrint("Stacktrace: $stacktrace");
    }
  }

  static Future<String> _getArchitecture() async {
    try {
      final result = await Process.run("getprop", ["ro.product.cpu.abi"]);
      final abi = result.stdout.toString().trim();
      if (abi.contains("arm64") || abi.contains("aarch64")) {
        return "aarch64";
      } else {
        return "x86_64";
      }
    } catch (_) {
      return "aarch64"; // Fallback to ARM64 as default
    }
  }

  static Future<void> _startApiServer(
    String prootPath,
    String rootfsPath,
  ) async {
    if (_apiProcess != null) {
      debugPrint("CodeDroid: API process is already active. Skipping start.");
      return;
    }

    debugPrint("CodeDroid: Starting API server...");
    try {
      final String canonicalProot = canonicalizePath(prootPath);
      final String canonicalRootfs = canonicalizePath(rootfsPath);
      final hostWorkingDir = Directory(canonicalRootfs).parent.path;
      
      final tmpDir = Directory("$hostWorkingDir/tmp");
      if (!tmpDir.existsSync()) {
        tmpDir.createSync(recursive: true);
      }
      final String tmpPath = tmpDir.resolveSymbolicLinksSync();

      final l2sDir = Directory("$canonicalRootfs/.l2s");
      if (!l2sDir.existsSync()) {
        l2sDir.createSync(recursive: true);
      }
      final String l2sPath = l2sDir.resolveSymbolicLinksSync();

      // Ensure a guest-writable Go temp directory exists inside rootfs
      final guestRootTmp = Directory("$canonicalRootfs/root/tmp");
      if (!guestRootTmp.existsSync()) {
        guestRootTmp.createSync(recursive: true);
      }

      // Clear stale apk lock before starting API server
      final lockFile = File("$canonicalRootfs/lib/apk/db/lock");
      if (lockFile.existsSync()) {
        try {
          lockFile.deleteSync();
          debugPrint(
            "CodeDroid: Stale apk lock cleared before API server startup.",
          );
        } catch (e) {
          debugPrint("CodeDroid: Failed to delete stale apk lock: $e");
        }
      }

      // Construct merged environment
      final Map<String, String> mergedEnv = buildCleanEnvironment(
        tmpPath: tmpPath,
        l2sPath: l2sPath,
        appendHostPath: true,
        extraEnv: {
          "GOTMPDIR": "/root/tmp",
          "CGO_ENABLED": "0",
          "NODE_OPTIONS": "--require /usr/local/lib/node_network_bypass.js",
        },
      );

      final List<String> args = [
        '-0',
        '--link2symlink',
        '-r',
        canonicalRootfs,
        '-w',
        '/',
        '-b',
        '/dev',
        '-b',
        '/proc',
        '-b',
        '/sys',
        '/usr/local/bin/codedroid_api',
      ];
      debugPrint("CodeDroid: Starting process: $canonicalProot with args: $args");

      _apiProcess = await Process.start(
        canonicalProot,
        args,
        workingDirectory: hostWorkingDir,
        environment: mergedEnv,
      );

      debugPrint(
        "CodeDroid: API Process started successfully with PID: ${_apiProcess!.pid}",
      );

      _apiProcess!.stdout.listen(
        (data) {
          final text = String.fromCharCodes(data);
          debugPrint("CodeDroid API stdout: $text");
          for (final line in text.split('\n')) {
            if (line.trim().isNotEmpty) {
              processLogs.add("[STDOUT] ${line.trim()}");
            }
          }
          if (processLogs.length > 2000) {
            processLogs.removeRange(0, processLogs.length - 2000);
          }
        },
        onError: (e) {
          debugPrint("CodeDroid API stdout error: $e");
          processLogs.add("[STDOUT ERROR] $e");
        },
        onDone: () {
          debugPrint("CodeDroid API stdout stream closed.");
          processLogs.add("[STDOUT] Stream closed.");
        },
      );

      _apiProcess!.stderr.listen(
        (data) {
          final text = String.fromCharCodes(data);
          debugPrint("CodeDroid API stderr: $text");
          for (final line in text.split('\n')) {
            if (line.trim().isNotEmpty) {
              processLogs.add("[STDERR] ${line.trim()}");
            }
          }
          if (processLogs.length > 2000) {
            processLogs.removeRange(0, processLogs.length - 2000);
          }
        },
        onError: (e) {
          debugPrint("CodeDroid API stderr error: $e");
          processLogs.add("[STDERR ERROR] $e");
        },
        onDone: () {
          debugPrint("CodeDroid API stderr stream closed.");
          processLogs.add("[STDERR] Stream closed.");
        },
      );

      _apiProcess!.exitCode.then((code) {
        debugPrint("CodeDroid API exited with code: $code");
        processLogs.add("[API PROCESS EXITED] Exit code: $code");
        _apiProcess = null;
      });
    } catch (e, stacktrace) {
      debugPrint("CodeDroid Error: Exception starting API server: $e");
      debugPrint("Stacktrace: $stacktrace");
      processLogs.add("[API START EXCEPTION] $e\n$stacktrace");
      _apiProcess = null;
    }
  }

  static Future<void> runGuestCommand(
    List<String> command,
    Function(String) onProgress,
  ) async {
    final appDir = await getApplicationSupportDirectory();
    final String appDirCanonical = canonicalizePath(appDir.path);
    final String linuxDir = "$appDirCanonical/linux";
    final String prootPath = canonicalizePath("$linuxDir/proot");
    final String rootfsPath = canonicalizePath("$linuxDir/rootfs");
    final String tmpPath = "$linuxDir/tmp";
    final String l2sPath = "$rootfsPath/.l2s";

    final mergedEnv = buildCleanEnvironment(
      tmpPath: tmpPath,
      l2sPath: l2sPath,
      appendHostPath: true,
    );

    final process = await Process.start(
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
        ...command,
      ],
      workingDirectory: linuxDir,
      environment: mergedEnv,
    );

    process.stdout.listen((data) {
      onProgress(String.fromCharCodes(data));
    });

    process.stderr.listen((data) {
      onProgress(String.fromCharCodes(data));
    });

    final exitCode = await process.exitCode;
    if (exitCode != 0) {
      throw Exception("Command failed with exit code $exitCode");
    }
  }

  static Future<void> runApkAdd(
    String packageName,
    Function(String) onProgress,
  ) async {
    final appDir = await getApplicationSupportDirectory();
    final String appDirCanonical = canonicalizePath(appDir.path);
    final String linuxDir = "$appDirCanonical/linux";
    final String prootPath = canonicalizePath("$linuxDir/proot");
    final String rootfsPath = canonicalizePath("$linuxDir/rootfs");

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
    mirrorRootfsDirectoriesToL2s(rootfsPath);
    cleanOrphanL2sMetadata(rootfsPath);
    await _makeRootfsWritable(rootfsPath);

    // Ensure critical guest directories exist inside rootfs
    for (final dirPath in ["$rootfsPath/tmp", "$rootfsPath/var/tmp", "$rootfsPath/root/tmp"]) {
      final dir = Directory(dirPath);
      if (!dir.existsSync()) {
        try {
          dir.createSync(recursive: true);
        } catch (_) {}
      }
    }

    // Clear stale apk lock before package installation
    final lockFile = File("$rootfsPath/lib/apk/db/lock");
    if (lockFile.existsSync()) {
      try {
        lockFile.deleteSync();
        onProgress("Cleared stale apk database lock.");
      } catch (_) {}
    }

    onProgress("Starting installer for $packageName...");

    final String trimmedName = packageName.trim();
    if (trimmedName == "go" || trimmedName.split(' ').contains("go")) {
      try {
        await installGoManually(rootfsPath, tmpPath, onProgress);
        _convertAbsoluteSymlinksToRelative(rootfsPath);
        onProgress("SUCCESS: Go and gopls installed successfully!");
        return;
      } catch (e) {
        onProgress("ERROR: Manual installation of Go failed: $e");
        return;
      }
    }

    if (trimmedName == "gcc" || trimmedName == "g++" || trimmedName == "build-base" || trimmedName.split(' ').contains("build-base")) {
      try {
        await installBuildBaseManually(rootfsPath, tmpPath, onProgress);
        try {
          onProgress("Installing C/C++ LSP support (clang-extra-tools)...");
          await runGuestCommand(["/sbin/apk", "add", "--no-cache", "clang-extra-tools"], onProgress);
        } catch (e) {
          onProgress("Warning: clang-extra-tools installation failed: $e");
        }
        _convertAbsoluteSymlinksToRelative(rootfsPath);
        onProgress("SUCCESS: build-base installed successfully!");
        return;
      } catch (e) {
        onProgress("ERROR: Manual installation of build-base failed: $e");
        return;
      }
    }

    if (trimmedName == "kotlin" || trimmedName.split(' ').contains("kotlin")) {
      try {
        await installKotlinManually(rootfsPath, tmpPath, onProgress);
        try {
          onProgress("Installing Kotlin LSP (kotlin-language-server)...");
          await runGuestCommand(["/bin/sh", "-c", "npm install -g kotlin-language-server || true"], onProgress);
        } catch (_) {}
        _convertAbsoluteSymlinksToRelative(rootfsPath);
        return;
      } catch (e) {
        onProgress("ERROR: Manual installation of kotlin failed: $e");
        return;
      }
    }

    if (trimmedName == "dart" || trimmedName.split(' ').contains("dart")) {
      try {
        await installDartManually(rootfsPath, tmpPath, onProgress);
        _convertAbsoluteSymlinksToRelative(rootfsPath);
        return;
      } catch (e) {
        onProgress("ERROR: Manual installation of dart failed: $e");
        return;
      }
    }

    // Pre-clean conflicting symlinks on the host to avoid PRoot rename/create errors
    await _cleanupConflictingSymlinks(rootfsPath, packageName);

    // Clean up any stale .apk.temp files left from previous failed runs
    _cleanupStaleApkTempFiles(rootfsPath);

    // Ensure write permissions on the rootfs and .l2s directory
    try {
      await Process.run("chmod", ["-R", "u+rwx", rootfsPath]);
      await Process.run("chmod", ["-R", "u+rwx", l2sPath]);
    } catch (_) {}

    final Map<String, List<String>> customCommands = {
      "python3": [
        "/bin/sh", "-c",
        "apk add --no-cache python3 py3-pip && pip3 install python-lsp-server black --break-system-packages"
      ],
      "nodejs npm": [
        "/bin/sh", "-c",
        "apk add --no-cache nodejs npm && npm install -g typescript typescript-language-server vscode-langservers-extracted"
      ],
      "rust cargo": [
        "/bin/sh", "-c",
        "apk add --no-cache rust cargo rust-analyzer"
      ],
      "go": [
        "/bin/sh", "-c",
        "apk add --no-cache go gopls"
      ],
      "openjdk17": [
        "/bin/sh", "-c",
        "apk add --no-cache openjdk17 maven gradle && npm install -g eclipse-jdt-ls || true"
      ],
      "csharp": [
        "/bin/sh", "-c",
        "apk add --no-cache dotnet7-sdk || apk add --no-cache dotnet-sdk-6.0 || apk add --no-cache dotnet8-sdk || apk add --no-cache dotnet-sdk"
      ],
      "swift": [
        "/bin/sh", "-c",
        "apk add --no-cache swift"
      ],
      "ruby": [
        "/bin/sh", "-c",
        "apk add --no-cache ruby ruby-dev build-base && gem install solargraph || true"
      ],
      "javascript": [
        "/bin/sh", "-c",
        "apk add --no-cache nodejs npm && npm install -g typescript typescript-language-server vscode-langservers-extracted"
      ],
      "typescript": [
        "/bin/sh", "-c",
        "apk add --no-cache nodejs npm && npm install -g typescript typescript-language-server vscode-langservers-extracted"
      ],
      "vanilla-js": [
        "/bin/sh", "-c",
        "apk add --no-cache nodejs npm && npm install -g typescript typescript-language-server vscode-langservers-extracted"
      ],
      "react": [
        "/bin/sh", "-c",
        "apk add --no-cache nodejs npm && npm install -g typescript typescript-language-server vscode-langservers-extracted @tailwindcss/language-server"
      ],
      "vue": [
        "/bin/sh", "-c",
        "apk add --no-cache nodejs npm && npm install -g @vue/language-server typescript"
      ],
      "svelte": [
        "/bin/sh", "-c",
        "apk add --no-cache nodejs npm && npm install -g svelte-language-server typescript"
      ],
      "angular": [
        "/bin/sh", "-c",
        "apk add --no-cache nodejs npm && npm install -g @angular/language-server typescript"
      ],
      "nextjs": [
        "/bin/sh", "-c",
        "apk add --no-cache nodejs npm && npm install -g typescript typescript-language-server vscode-langservers-extracted @tailwindcss/language-server"
      ],
      "remix": [
        "/bin/sh", "-c",
        "apk add --no-cache nodejs npm && npm install -g typescript typescript-language-server vscode-langservers-extracted"
      ],
      "git": [
        "/bin/sh", "-c",
        "apk add --no-cache git"
      ]
    };

    try {
      final List<String> guestCommand = customCommands[trimmedName] ?? [
        '/sbin/apk',
        'add',
        '--no-cache',
        '--force-overwrite',
        ...packageName.split(' '),
      ];

      // Determine the packages to protect during sanitization
      final List<String> currentPackages = [];
      if (trimmedName == "python3") {
        currentPackages.addAll(["python3", "py3-pip"]);
      } else if (trimmedName == "nodejs npm" || trimmedName == "javascript" || trimmedName == "typescript" || trimmedName == "vanilla-js" || trimmedName == "react" || trimmedName == "vue" || trimmedName == "svelte" || trimmedName == "angular" || trimmedName == "nextjs" || trimmedName == "remix") {
        currentPackages.addAll(["nodejs", "npm"]);
      } else if (trimmedName == "rust cargo") {
        currentPackages.addAll(["rust", "cargo", "rust-analyzer"]);
      } else if (trimmedName == "go") {
        currentPackages.addAll(["go", "gopls"]);
      } else if (trimmedName == "openjdk17") {
        currentPackages.addAll(["openjdk17", "maven", "gradle"]);
      } else if (trimmedName == "csharp") {
        currentPackages.addAll(["dotnet7-sdk", "dotnet-sdk-6.0", "dotnet8-sdk", "dotnet-sdk"]);
      } else if (trimmedName == "swift") {
        currentPackages.addAll(["swift"]);
      } else if (trimmedName == "ruby") {
        currentPackages.addAll(["ruby", "ruby-dev", "build-base"]);
      } else if (trimmedName == "git") {
        currentPackages.addAll(["git"]);
      } else {
        currentPackages.addAll(packageName.split(' '));
      }

      _sanitizeApkWorld(rootfsPath, currentPackages);

      final process = await Process.start(
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
          ...guestCommand,
        ],
        workingDirectory: linuxDir,
        environment: buildCleanEnvironment(
          tmpPath: tmpPath,
          l2sPath: l2sPath,
        ),
      );

      process.stdout.listen((data) {
        onProgress(String.fromCharCodes(data));
      });

      process.stderr.listen((data) {
        onProgress(String.fromCharCodes(data));
      });

      final exitCode = await process.exitCode;
      if (exitCode == 0) {
        onProgress("SUCCESS: $packageName installed successfully!");
        _convertAbsoluteSymlinksToRelative(rootfsPath);
      } else {
        onProgress("ERROR: Installation failed with exit code $exitCode");
        _sanitizeApkWorld(rootfsPath, []);
      }
    } catch (e) {
      onProgress("ERROR: Failed to run installer: $e");
      _sanitizeApkWorld(rootfsPath, []);
    }
  }

  static bool _deleteFileOrLink(String path) {
    try {
      final link = Link(path);
      if (link.existsSync()) {
        link.deleteSync();
        return true;
      }
    } catch (_) {}
    try {
      final file = File(path);
      if (file.existsSync()) {
        file.deleteSync();
        return true;
      }
    } catch (_) {}
    return false;
  }

  static Future<void> deletePackage(
    String packageName,
    Function(String) onProgress,
  ) async {
    final appDir = await getApplicationSupportDirectory();
    final String appDirCanonical = canonicalizePath(appDir.path);
    final String linuxDir = "$appDirCanonical/linux";
    final String rootfsPath = canonicalizePath("$linuxDir/rootfs");
    await _makeRootfsWritable(rootfsPath);

    // Clear stale apk lock before package removal
    final lockFile = File("$rootfsPath/lib/apk/db/lock");
    if (lockFile.existsSync()) {
      try {
        lockFile.deleteSync();
        onProgress("Cleared stale apk database lock.");
      } catch (_) {}
    }

    final trimmedName = packageName.trim();
    onProgress("Starting removal for $trimmedName...");

    if (trimmedName == "go" || trimmedName.split(' ').contains("go")) {
      // 1. Delete Go SDK directory
      final goDir = Directory("$rootfsPath/usr/lib/go");
      if (goDir.existsSync()) {
        try {
          goDir.deleteSync(recursive: true);
          onProgress("Deleted /usr/lib/go directory.");
        } catch (e) {
          onProgress("Warning: Failed to delete /usr/lib/go: $e");
        }
      }
      // 2. Delete binaries/symlinks
      for (final file in ["go", "gofmt", "gopls"]) {
        final path = "$rootfsPath/usr/bin/$file";
        if (_deleteFileOrLink(path)) {
          onProgress("Deleted /usr/bin/$file.");
        }
      }
      // 3. Try apk del just in case
      try {
        await runGuestCommand(["/sbin/apk", "del", "go", "gopls"], onProgress);
      } catch (_) {}
      onProgress("SUCCESS: Go removed successfully!");
      return;
    }

    if (trimmedName == "kotlin" || trimmedName.split(' ').contains("kotlin")) {
      // 1. Delete Kotlin share directory
      final kotlinDir = Directory("$rootfsPath/usr/share/kotlin");
      if (kotlinDir.existsSync()) {
        try {
          kotlinDir.deleteSync(recursive: true);
          onProgress("Deleted /usr/share/kotlin directory.");
        } catch (e) {
          onProgress("Warning: Failed to delete /usr/share/kotlin: $e");
        }
      }
      // 2. Delete binaries/symlinks
      for (final file in ["kotlin", "kotlinc"]) {
        final path = "$rootfsPath/usr/bin/$file";
        if (_deleteFileOrLink(path)) {
          onProgress("Deleted /usr/bin/$file.");
        }
      }
      // Also run npm uninstall of the language server if installed
      try {
        await runGuestCommand(["/bin/sh", "-c", "npm uninstall -g kotlin-language-server || true"], onProgress);
      } catch (_) {}
      onProgress("SUCCESS: Kotlin removed successfully!");
      return;
    }

    if (trimmedName == "dart" || trimmedName.split(' ').contains("dart")) {
      // 1. Delete Dart SDK directory
      final dartDir = Directory("$rootfsPath/usr/lib/dart");
      if (dartDir.existsSync()) {
        try {
          dartDir.deleteSync(recursive: true);
          onProgress("Deleted /usr/lib/dart directory.");
        } catch (e) {
          onProgress("Warning: Failed to delete /usr/lib/dart: $e");
        }
      }
      // 2. Delete Dart binary/symlink
      final path = "$rootfsPath/usr/bin/dart";
      if (_deleteFileOrLink(path)) {
        onProgress("Deleted /usr/bin/dart.");
      }
      onProgress("SUCCESS: Dart removed successfully!");
      return;
    }

    if (trimmedName == "gcc" || trimmedName == "g++" || trimmedName == "build-base" || trimmedName.split(' ').contains("build-base")) {
      // Delete main C/C++ binaries
      for (final file in ["gcc", "g++", "cpp", "cc", "c++", "make", "clangd"]) {
        final path = "$rootfsPath/usr/bin/$file";
        if (_deleteFileOrLink(path)) {
          onProgress("Deleted /usr/bin/$file.");
        }
      }
      // Try to uninstall any apk installed clang/lsp tools
      try {
        await runGuestCommand(["/sbin/apk", "del", "clang-extra-tools"], onProgress);
      } catch (_) {}
      onProgress("SUCCESS: C/C++ build tools removed successfully!");
      return;
    }

    if (trimmedName == "csharp" || trimmedName.split(' ').contains("csharp")) {
      try {
        final installed = _getInstalledApkPackages(rootfsPath);
        final dotnetPkgs = installed.where((pkg) => pkg.contains("dotnet") || pkg.contains("aspnetcore")).toList();
        if (dotnetPkgs.isNotEmpty) {
          onProgress("Deleting installed dotnet packages: ${dotnetPkgs.join(', ')}");
          // Include potential metapackage/explicit packages just in case
          final pkgsToDelete = {...dotnetPkgs, "dotnet7-sdk", "dotnet-sdk-6.0", "dotnet8-sdk", "dotnet-sdk"}.toList();
          await runGuestCommand(["/sbin/apk", "del", ...pkgsToDelete], onProgress);
        } else {
          await runGuestCommand(["/sbin/apk", "del", "dotnet7-sdk", "dotnet-sdk-6.0", "dotnet8-sdk", "dotnet-sdk"], onProgress);
        }
        final path = "$rootfsPath/usr/bin/dotnet";
        if (_deleteFileOrLink(path)) {
          onProgress("Deleted /usr/bin/dotnet.");
        }
      } catch (e) {
        onProgress("Warning: Error during dotnet packages removal: $e");
      }
      onProgress("SUCCESS: C# removed successfully!");
      return;
    }

    // For all other packages, use normal apk del
    try {
      onProgress("Running apk del for $packageName...");
      String pkgs = packageName;
      if (packageName == "javascript") {
        pkgs = "nodejs npm";
      } else if (packageName == "openjdk17") {
        pkgs = "openjdk17 maven gradle";
      }
      final List<String> cmdArgs = ["/sbin/apk", "del", "--no-cache"];
      cmdArgs.addAll(pkgs.split(' '));
      await runGuestCommand(cmdArgs, onProgress);
      onProgress("SUCCESS: $packageName removed successfully!");
    } catch (e) {
      onProgress("ERROR: Failed to remove $packageName: $e");
    }
  }

  static Future<void> stop() async {
    if (_apiProcess != null) {
      _apiProcess!.kill();
      _apiProcess = null;
    }
  }

  static Future<void> _killStaleProcesses() async {
    debugPrint("CodeDroid: Cleaning up any stale background processes...");
    try {
      // Try pkill first to terminate any existing codedroid_api or proot instances
      await Process.run("pkill", ["-f", "codedroid_api"]);
      await Process.run("pkill", ["-f", "proot"]);
    } catch (_) {}
    try {
      // Fallback/additional killall
      await Process.run("killall", ["codedroid_api"]);
      await Process.run("killall", ["proot"]);
    } catch (_) {}
    // Give the OS a short moment to release the network ports and file locks
    await Future.delayed(const Duration(milliseconds: 500));
  }

  static void _convertAbsoluteSymlinksToRelative(String rootfsPath) {
    debugPrint("CodeDroid: Scanning and converting absolute symlinks to relative...");
    final rootfsDir = Directory(rootfsPath);
    if (!rootfsDir.existsSync()) return;

    try {
      final List<FileSystemEntity> entities = rootfsDir.listSync(recursive: true, followLinks: false);
      int count = 0;
      for (final entity in entities) {
        if (entity is Link) {
          try {
            final target = entity.targetSync();
            if (target.startsWith('/')) {
              // It is an absolute symlink (e.g. "/bin/busybox")
              final linkPath = entity.path;
              // Remove the rootfsPath prefix to get the guest link path
              final relativeLinkPath = linkPath.substring(rootfsPath.length);
              
              // Count directory levels in relativeLinkPath to go up to guest rootfs '/'
              final components = relativeLinkPath.split('/');
              int levels = components.length - 2;
              if (levels < 0) levels = 0;
              
              String prefix = '';
              if (levels == 0) {
                prefix = '.';
              } else {
                prefix = List.generate(levels, (_) => '..').join('/');
              }
              
              // Target path relative to rootfs root
              final relativeTarget = target.startsWith('/') ? target.substring(1) : target;
              final newTarget = "$prefix/$relativeTarget";
              
              // Delete the old link and create a new one pointing to newTarget
              entity.deleteSync();
              final newLink = Link(linkPath);
              newLink.createSync(newTarget);
              count++;
            }
          } catch (e) {
            // Ignore individual link errors
          }
        }
      }
      debugPrint("CodeDroid: Converted $count absolute symlinks to relative.");
    } catch (e) {
      debugPrint("CodeDroid Error during symlink conversion: $e");
    }
  }

  static Future<void> _cleanupConflictingSymlinks(String rootfsPath, String packageName) async {
    final arch = await _getArchitecture();
    final targetTuple = "$arch-alpine-linux-musl";
    final name = packageName.toLowerCase();
    final List<String> filesToDelete = [];

    if (name.contains("go") || name.contains("gopls")) {
      try {
        Directory("$rootfsPath/usr/bin").createSync(recursive: true);
        Directory("$rootfsPath/usr/lib/go").createSync(recursive: true);
        Directory("$rootfsPath/usr/$targetTuple/bin").createSync(recursive: true);
      } catch (e) {
        debugPrint("CodeDroid Warning: Failed to create directory structure for Go: $e");
      }

      final List<String> dirsToCreate = [
        "$rootfsPath/usr/bin",
        "$rootfsPath/usr/lib/go",
        "$rootfsPath/usr/$targetTuple/bin",
        "$rootfsPath/.l2s/usr/bin",
        "$rootfsPath/.l2s/usr/lib/go",
        "$rootfsPath/.l2s/usr/$targetTuple/bin",
      ];
      for (final d in dirsToCreate) {
        try { Directory(d).createSync(recursive: true); } catch (_) {}
      }

      filesToDelete.addAll([
        "usr/bin/go",
        "usr/bin/gofmt",
        "usr/bin/gopls",
        "usr/bin/ld.gold",
        "usr/$targetTuple/bin/ld.gold",
      ]);
    }

    if (name.contains("build-base") || 
        name.contains("binutils") || 
        name.contains("gcc") || 
        name.contains("g++") || 
        name.contains("clang") || 
        name.contains("make")) {
      // Ensure target toolchain directories exist on the host filesystem first
      try {
        Directory("$rootfsPath/usr/bin").createSync(recursive: true);
        Directory("$rootfsPath/usr/$targetTuple/bin").createSync(recursive: true);
      } catch (e) {
        debugPrint("CodeDroid Warning: Failed to create directory structure: $e");
      }

      // Pre-create ALL directories apk will write into, both in rootfs and .l2s
      final List<String> dirsToCreate = [
        "$rootfsPath/usr/bin",
        "$rootfsPath/usr/lib",
        "$rootfsPath/usr/libexec",
        "$rootfsPath/usr/$targetTuple/bin",
        "$rootfsPath/usr/$targetTuple/lib",
        "$rootfsPath/usr/lib/gcc/$targetTuple",
        "$rootfsPath/usr/lib/gcc/$targetTuple/12.2.1",
        "$rootfsPath/.l2s/usr/bin",
        "$rootfsPath/.l2s/usr/lib",
        "$rootfsPath/.l2s/usr/libexec",
        "$rootfsPath/.l2s/usr/$targetTuple/bin",
        "$rootfsPath/.l2s/usr/$targetTuple/lib",
        "$rootfsPath/.l2s/usr/lib/gcc/$targetTuple",
        "$rootfsPath/.l2s/usr/lib/gcc/$targetTuple/12.2.1",
      ];
      for (final d in dirsToCreate) {
        try { Directory(d).createSync(recursive: true); } catch (_) {}
      }

      filesToDelete.addAll([
        "usr/bin/ar",
        "usr/bin/as",
        "usr/bin/ld",
        "usr/bin/ld.bfd",
        "usr/bin/nm",
        "usr/bin/objcopy",
        "usr/bin/objdump",
        "usr/bin/ranlib",
        "usr/bin/readelf",
        "usr/bin/strip",
        "usr/bin/gcc",
        "usr/bin/g++",
        "usr/bin/c++",
        "usr/bin/cpp",
        "usr/bin/gcc-ar",
        "usr/bin/gcc-nm",
        "usr/bin/gcc-ranlib",
        "usr/bin/$targetTuple-gcc-12.2.1",
        "usr/bin/$targetTuple-g++",
        "usr/bin/$targetTuple-ld",
        "usr/bin/$targetTuple-ld.bfd",
        "usr/bin/$targetTuple-ar",
        "usr/bin/$targetTuple-as",
        "usr/bin/$targetTuple-nm",
        "usr/bin/$targetTuple-objcopy",
        "usr/bin/$targetTuple-objdump",
        "usr/bin/$targetTuple-ranlib",
        "usr/bin/$targetTuple-strip",
        "usr/$targetTuple/bin/ar",
        "usr/$targetTuple/bin/as",
        "usr/$targetTuple/bin/ld",
        "usr/$targetTuple/bin/ld.bfd",
        "usr/$targetTuple/bin/nm",
        "usr/$targetTuple/bin/objcopy",
        "usr/$targetTuple/bin/objdump",
        "usr/$targetTuple/bin/ranlib",
        "usr/$targetTuple/bin/strip",
      ]);
    }

    for (final relPath in filesToDelete) {
      final path = "$rootfsPath/$relPath";
      final file = File(path);
      final link = Link(path);
      
      // Explicitly delete corresponding .l2s backing file if it exists
      final l2sPath = "$rootfsPath/.l2s/$relPath";
      final l2sFile = File(l2sPath);
      if (l2sFile.existsSync()) {
        try {
          l2sFile.deleteSync();
          debugPrint("CodeDroid: Deleted l2s backing file: $l2sPath");
        } catch (_) {}
      }

      try {
        if (link.existsSync()) {
          // If the link points to a link2symlink backing file in .l2s, delete that first
          final target = link.targetSync();
          if (target.contains('.l2s')) {
            final linkUri = Uri.file(link.path);
            final backingPath = linkUri.resolve(target).toFilePath();
            final backingFile = File(backingPath);
            if (backingFile.existsSync()) {
              backingFile.deleteSync();
              debugPrint("CodeDroid: Deleted l2s backing file: $backingPath");
            }
          }
          link.deleteSync();
          debugPrint("CodeDroid: Deleted conflicting symlink: $relPath");
        } else if (file.existsSync()) {
          file.deleteSync();
          debugPrint("CodeDroid: Deleted conflicting file: $relPath");
        }
      } catch (e) {
        debugPrint("CodeDroid Warning: Failed to delete $relPath: $e");
      }
    }
  }

  static void _cleanupStaleApkTempFiles(String rootfsPath) {
    debugPrint("CodeDroid: Cleaning up any stale apk temp files...");
    final pathsToScan = [rootfsPath, "$rootfsPath/.l2s"];

    for (final scanPath in pathsToScan) {
      final dir = Directory(scanPath);
      if (!dir.existsSync()) continue;

      try {
        final List<FileSystemEntity> entities = dir.listSync(recursive: true, followLinks: false);
        for (final entity in entities) {
          final name = entity.path.split('/').last;
          if (name.contains('.apk.')) {
            try {
              if (entity is Link) {
                final target = entity.targetSync();
                if (target.contains('.l2s')) {
                  final linkUri = Uri.file(entity.path);
                  final backingPath = linkUri.resolve(target).toFilePath();
                  final backingFile = File(backingPath);
                  if (backingFile.existsSync()) {
                    backingFile.deleteSync();
                  }
                }
              }
              entity.deleteSync();
              debugPrint("CodeDroid: Deleted stale apk temp file: ${entity.path}");
            } catch (_) {}
          }
        }
      } catch (e) {
        debugPrint("CodeDroid Warning: Failed to clean stale apk temp files in $scanPath: $e");
      }
    }
  }

  static String canonicalizePath(String path) {
    try {
      final file = File(path);
      if (file.existsSync()) {
        return file.resolveSymbolicLinksSync();
      }
      final dir = Directory(path);
      if (dir.existsSync()) {
        return dir.resolveSymbolicLinksSync();
      }
      // If the path does not exist, canonicalize its parent and join the basename
      final parent = dir.parent;
      if (parent.existsSync()) {
        return "${parent.resolveSymbolicLinksSync()}/${path.split('/').last}";
      }
    } catch (_) {}
    return path;
  }

  static Map<String, String> buildCleanEnvironment({
    required String tmpPath,
    required String l2sPath,
    Map<String, String>? extraEnv,
    bool appendHostPath = false,
  }) {
    final Map<String, String> env = Map<String, String>.from(Platform.environment);
    // Remove environment variables that might interfere with guest/chroot binaries
    env.remove("LD_PRELOAD");
    env.remove("LD_LIBRARY_PATH");

    final String baseGuestPath = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
    if (appendHostPath) {
      final String hostPath = Platform.environment['PATH'] ?? '';
      env["PATH"] = "$baseGuestPath:$hostPath";
    } else {
      env["PATH"] = baseGuestPath;
    }

    env["HOME"] = "/root";
    env["USER"] = "root";
    env["TERM"] = "xterm-256color";
    env["PROOT_TMP_DIR"] = tmpPath;
    env["PROOT_L2S_DIR"] = l2sPath;
    env["TMPDIR"] = "/tmp";
    env["TMP"] = "/tmp";
    env["TEMP"] = "/tmp";

    if (extraEnv != null) {
      env.addAll(extraEnv);
    }
    return env;
  }

  static void cleanOrphanL2sMetadata(String rootfsPath) {
    debugPrint("CodeDroid: Cleaning orphan .l2s metadata entries...");
    final l2sDirPath = "$rootfsPath/.l2s";
    final l2sDir = Directory(l2sDirPath);
    if (!l2sDir.existsSync()) return;

    try {
      final List<FileSystemEntity> entities = l2sDir.listSync(recursive: true, followLinks: false);
      for (final entity in entities) {
        if (entity is File || entity is Link) {
          final relPath = entity.path.substring(l2sDirPath.length);
          if (relPath.isEmpty) continue;

          final rootfsFile = File("$rootfsPath$relPath");
          final rootfsLink = Link("$rootfsPath$relPath");
          final rootfsDir = Directory("$rootfsPath$relPath");

          if (!rootfsFile.existsSync() && !rootfsLink.existsSync() && !rootfsDir.existsSync()) {
            try {
              entity.deleteSync();
              debugPrint("CodeDroid: Deleted orphan .l2s entry: ${entity.path}");
            } catch (_) {}
          }
        }
      }
    } catch (e) {
      debugPrint("CodeDroid Warning: Failed to clean orphan .l2s metadata: $e");
    }
  }

  static void mirrorRootfsDirectoriesToL2s(String rootfsPath) {
    debugPrint("CodeDroid: Mirroring rootfs directory structure to .l2s...");
    final rootfsDir = Directory(rootfsPath);
    if (!rootfsDir.existsSync()) return;

    final l2sDirPath = "$rootfsPath/.l2s";
    try {
      final List<FileSystemEntity> entities = rootfsDir.listSync(recursive: true, followLinks: false);
      for (final entity in entities) {
        if (entity is Directory) {
          final relPath = entity.path.substring(rootfsPath.length);
          if (relPath.isEmpty || relPath.startsWith('/.l2s')) continue;
          
          final targetL2sDir = Directory("$l2sDirPath$relPath");
          if (!targetL2sDir.existsSync()) {
            targetL2sDir.createSync(recursive: true);
          }
        }
      }
    } catch (e) {
      debugPrint("CodeDroid Warning: Failed to mirror directories to .l2s: $e");
    }
  }

  static Future<void> installBuildBaseManually(
    String rootfsPath,
    String tmpPath,
    Function(String) onProgress,
  ) async {
    final packages = [
      "zstd-libs-1.5.5-r4.apk",
      "musl-1.2.4-r3.apk",
      "libgcc-12.2.1_git20220924-r10.apk",
      "libstdc++-12.2.1_git20220924-r10.apk",
      "libgomp-12.2.1_git20220924-r10.apk",
      "libatomic-12.2.1_git20220924-r10.apk",
      "gmp-6.2.1-r3.apk",
      "mpfr4-4.2.0_p12-r0.apk",
      "mpc1-1.3.1-r1.apk",
      "binutils-2.40-r8.apk",
      "gcc-12.2.1_git20220924-r10.apk",
      "musl-dev-1.2.4-r3.apk",
      "libc-dev-0.7.2-r5.apk",
      "g++-12.2.1_git20220924-r10.apk",
      "make-4.4.1-r1.apk",
      "fortify-headers-1.1-r3.apk",
    ];

    final String arch = await _getArchitecture();
    final baseUrl = "https://dl-cdn.alpinelinux.org/alpine/v3.18/main/$arch/";

    // Clean up tmp directory first to reclaim space
    final tmpDir = Directory(tmpPath);
    if (tmpDir.existsSync()) {
      try {
        final list = tmpDir.listSync();
        for (var file in list) {
          try { file.deleteSync(recursive: true); } catch (_) {}
        }
      } catch (_) {}
    }

    onProgress("Starting manual installation of build-base toolchain...");

    for (final pkg in packages) {
      final name = pkg.split('-').first;
      onProgress("[$name] Downloading...");
      final apkUrl = "$baseUrl$pkg";
      final apkPath = "$tmpPath/$pkg";
      final tarPath = "$tmpPath/${pkg.replaceAll('.apk', '.tar')}";

      try {
        await _downloadFile(apkUrl, apkPath);
        onProgress("[$name] Decompressing...");
        await _decompressApk(apkPath, tarPath);
        onProgress("[$name] Extracting files...");
        await TarExtractor.extract(tarPath, rootfsPath);
        onProgress("[$name] Installation done!");
      } catch (e) {
        onProgress("ERROR: Failed to install $name: $e");
        rethrow;
      } finally {
        try {
          final fApk = File(apkPath);
          if (fApk.existsSync()) fApk.deleteSync();
          final fTar = File(tarPath);
          if (fTar.existsSync()) fTar.deleteSync();
        } catch (_) {}
      }
    }
  }
  static Future<void> installGoManually(
    String rootfsPath,
    String tmpPath,
    Function(String) onProgress,
  ) async {
    final packages = [
      "binutils-gold-2.40-r8.apk",
      "go-1.20.11-r0.apk",
      "gopls-0.11.0-r8.apk",
    ];

    // Clean up tmp directory first to reclaim space
    final tmpDir = Directory(tmpPath);
    if (tmpDir.existsSync()) {
      try {
        final list = tmpDir.listSync();
        for (var file in list) {
          try { file.deleteSync(recursive: true); } catch (_) {}
        }
      } catch (_) {}
    }

    final String arch = await _getArchitecture();

    onProgress("Cleaning up conflicting symlinks...");
    await _cleanupConflictingSymlinks(rootfsPath, "go");

    onProgress("Starting manual installation of Go compiler and gopls LSP...");

    try {
      onProgress("Installing shared library dependencies (libgcc, libstdc++, zstd-libs, zlib)...");
      await runGuestCommand(["/sbin/apk", "add", "--no-cache", "libgcc", "libstdc++", "zstd-libs", "zlib"], onProgress);
    } catch (e) {
      onProgress("Warning: Failed to pre-install dependencies via apk: $e. Proceeding with extraction...");
    }

    for (final pkg in packages) {
      final name = pkg.split('-').first;
      final apkPath = "$tmpPath/$pkg";
      final tarPath = "$tmpPath/${pkg.replaceAll('.apk', '.tar')}";

      final guestApkFile = File("$rootfsPath/tmp/go-apks/$pkg");
      if (guestApkFile.existsSync()) {
        onProgress("[$name] Found pre-downloaded package in cache.");
        try {
          guestApkFile.copySync(apkPath);
        } catch (copyErr) {
          onProgress("[$name] Cache copy failed ($copyErr), downloading...");
          final baseUrl = (name == "gopls" || name == "go") 
              ? "https://dl-cdn.alpinelinux.org/alpine/v3.18/community/$arch/"
              : "https://dl-cdn.alpinelinux.org/alpine/v3.18/main/$arch/";
          await _downloadFile("$baseUrl$pkg", apkPath);
        }
      } else {
        onProgress("[$name] Downloading...");
        final baseUrl = (name == "gopls" || name == "go") 
            ? "https://dl-cdn.alpinelinux.org/alpine/v3.18/community/$arch/"
            : "https://dl-cdn.alpinelinux.org/alpine/v3.18/main/$arch/";
        try {
          await _downloadFile("$baseUrl$pkg", apkPath);
        } catch (e) {
          onProgress("ERROR: Failed to download $name: $e");
          rethrow;
        }
      }

      try {
        onProgress("[$name] Decompressing...");
        await _decompressApk(apkPath, tarPath);
        onProgress("[$name] Extracting files...");
        await TarExtractor.extract(tarPath, rootfsPath);
        onProgress("[$name] Installation done!");
      } catch (e) {
        onProgress("ERROR: Failed to extract $name: $e");
        rethrow;
      } finally {
        try {
          final fApk = File(apkPath);
          if (fApk.existsSync()) fApk.deleteSync();
          final fTar = File(tarPath);
          if (fTar.existsSync()) fTar.deleteSync();
        } catch (_) {}
      }
    }
  }

  static Future<void> _downloadFile(String url, String savePath) async {
    final client = HttpClient();
    try {
      final request = await client.getUrl(Uri.parse(url));
      final response = await request.close();
      if (response.statusCode == 200) {
        final file = File(savePath);
        final sink = file.openWrite();
        await response.pipe(sink);
        await sink.close();
      } else {
        throw Exception("Status code ${response.statusCode}");
      }
    } finally {
      client.close();
    }
  }

  static Future<void> _decompressApk(String apkPath, String tarPath) async {
    final result = await Process.run("sh", [
      "-c",
      "gzip -d -c -f '$apkPath' > '$tarPath'"
    ]);
    if (result.exitCode != 0) {
      throw Exception("Exit code ${result.exitCode}: ${result.stderr}");
    }
  }

  static Future<void> installKotlinManually(
    String rootfsPath,
    String tmpPath,
    Function(String) onProgress,
  ) async {
    onProgress("Starting manual installation of Kotlin compiler...");

    // Clean up tmp directory first
    final tmpDir = Directory(tmpPath);
    if (tmpDir.existsSync()) {
      try {
        final list = tmpDir.listSync();
        for (var file in list) {
          try { file.deleteSync(recursive: true); } catch (_) {}
        }
      } catch (_) {}
    }

    onProgress("Installing Java JDK dependency (openjdk17) via apk...");
    final appDir = await getApplicationSupportDirectory();
    final String appDirCanonical = canonicalizePath(appDir.path);
    final String linuxDir = "$appDirCanonical/linux";
    final String prootPath = canonicalizePath("$linuxDir/proot");
    final String l2sPath = "$rootfsPath/.l2s";

    final result = await Process.run(
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
        '/sbin/apk',
        'add',
        '--no-cache',
        'openjdk17',
        'maven',
        'gradle',
      ],
      workingDirectory: linuxDir,
      environment: buildCleanEnvironment(
        tmpPath: "$linuxDir/tmp",
        l2sPath: l2sPath,
      ),
    );

    if (result.exitCode != 0) {
      onProgress("Warning: Java JDK installation failed: ${result.stderr}");
    } else {
      onProgress("Java JDK dependency installed successfully.");
    }

    final kotlinVersion = "1.9.24";
    final kotlinUrl = "https://github.com/JetBrains/kotlin/releases/download/v$kotlinVersion/kotlin-compiler-$kotlinVersion.zip";
    final kotlinZipPath = "$tmpPath/kotlin-compiler.zip";

    try {
      onProgress("Downloading Kotlin compiler zip...");
      await _downloadFile(kotlinUrl, kotlinZipPath);

      onProgress("Extracting Kotlin compiler...");
      final kotlinDestDir = Directory("$rootfsPath/usr/share/kotlin");
      if (kotlinDestDir.existsSync()) {
        try {
          kotlinDestDir.deleteSync(recursive: true);
        } catch (_) {}
      }
      kotlinDestDir.createSync(recursive: true);

      final unzipResult = await Process.run("unzip", [
        "-o",
        kotlinZipPath,
        "-d",
        "$rootfsPath/usr/share/kotlin"
      ]);

      if (unzipResult.exitCode != 0) {
        throw Exception("Failed to unzip Kotlin compiler: ${unzipResult.stderr}");
      }

      onProgress("Creating Kotlin executable symlinks...");
      final binDir = Directory("$rootfsPath/usr/bin");
      if (!binDir.existsSync()) binDir.createSync(recursive: true);

      final kotlincLink = Link("$rootfsPath/usr/bin/kotlinc");
      if (kotlincLink.existsSync()) {
        try {
          kotlincLink.deleteSync();
        } catch (_) {}
      }
      await kotlincLink.create("../share/kotlin/kotlinc/bin/kotlinc");

      final kotlinLink = Link("$rootfsPath/usr/bin/kotlin");
      if (kotlinLink.existsSync()) {
        try {
          kotlinLink.deleteSync();
        } catch (_) {}
      }
      await kotlinLink.create("../share/kotlin/kotlinc/bin/kotlin");

      try {
        await Process.run("chmod", ["+x", "$rootfsPath/usr/share/kotlin/kotlinc/bin/kotlin"]);
        await Process.run("chmod", ["+x", "$rootfsPath/usr/share/kotlin/kotlinc/bin/kotlinc"]);
      } catch (_) {}
    } finally {
      try {
        final file = File(kotlinZipPath);
        if (file.existsSync()) {
          file.deleteSync();
        }
      } catch (_) {}
    }

    onProgress("SUCCESS: Kotlin compiler installed successfully!");
  }

  static Future<void> installDartManually(
    String rootfsPath,
    String tmpPath,
    Function(String) onProgress,
  ) async {
    onProgress("Starting manual installation of Dart SDK...");

    // Clean up tmp directory first
    final tmpDir = Directory(tmpPath);
    if (tmpDir.existsSync()) {
      try {
        final list = tmpDir.listSync();
        for (var file in list) {
          try { file.deleteSync(recursive: true); } catch (_) {}
        }
      } catch (_) {}
    }

    onProgress("Installing glibc compatibility layer (gcompat) via apk...");
    final appDir = await getApplicationSupportDirectory();
    final String appDirCanonical = canonicalizePath(appDir.path);
    final String linuxDir = "$appDirCanonical/linux";
    final String prootPath = canonicalizePath("$linuxDir/proot");
    final String l2sPath = "$rootfsPath/.l2s";

    final result = await Process.run(
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
        '/sbin/apk',
        'add',
        '--no-cache',
        'gcompat',
      ],
      workingDirectory: linuxDir,
      environment: buildCleanEnvironment(
        tmpPath: "$linuxDir/tmp",
        l2sPath: l2sPath,
      ),
    );

    if (result.exitCode != 0) {
      onProgress("Warning: gcompat installation failed: ${result.stderr}");
    } else {
      onProgress("glibc compatibility layer (gcompat) installed successfully.");
    }

    final dartVersion = "3.4.3";
    final dartUrl = "https://storage.googleapis.com/dart-archive/channels/stable/release/$dartVersion/sdk/dartsdk-linux-arm64-release.zip";
    final dartZipPath = "$tmpPath/dartsdk.zip";

    try {
      onProgress("Downloading Dart SDK zip...");
      await _downloadFile(dartUrl, dartZipPath);

      onProgress("Extracting Dart SDK...");
      final dartDestDir = Directory("$rootfsPath/usr/lib/dart");
      if (dartDestDir.existsSync()) {
        try {
          dartDestDir.deleteSync(recursive: true);
        } catch (_) {}
      }
      dartDestDir.createSync(recursive: true);

      final unzipResult = await Process.run("unzip", [
        "-o",
        dartZipPath,
        "-d",
        "$rootfsPath/usr/lib/dart"
      ]);

      if (unzipResult.exitCode != 0) {
        throw Exception("Failed to unzip Dart SDK: ${unzipResult.stderr}");
      }

      onProgress("Creating Dart executable symlink...");
      final binDir = Directory("$rootfsPath/usr/bin");
      if (!binDir.existsSync()) binDir.createSync(recursive: true);

      final dartLink = Link("$rootfsPath/usr/bin/dart");
      if (dartLink.existsSync()) {
        try {
          dartLink.deleteSync();
        } catch (_) {}
      }
      await dartLink.create("../lib/dart/dart-sdk/bin/dart");

      try {
        await Process.run("chmod", ["+x", "$rootfsPath/usr/lib/dart/dart-sdk/bin/dart"]);
      } catch (_) {}
    } finally {
      try {
        final file = File(dartZipPath);
        if (file.existsSync()) {
          file.deleteSync();
        }
      } catch (_) {}
    }

    onProgress("SUCCESS: Dart SDK installed successfully!");
  }

  static Set<String> _getInstalledApkPackages(String rootfsPath) {
    final Set<String> installed = {};
    final dbFile = File("$rootfsPath/lib/apk/db/installed");
    if (!dbFile.existsSync()) return installed;

    try {
      final lines = dbFile.readAsLinesSync();
      for (final line in lines) {
        if (line.startsWith("P:")) {
          installed.add(line.substring(2).trim());
        }
      }
    } catch (_) {}
    return installed;
  }

  static void _sanitizeApkWorld(String rootfsPath, List<String> currentPackages) {
    final worldFile = File("$rootfsPath/etc/apk/world");
    if (!worldFile.existsSync()) return;

    try {
      final installed = _getInstalledApkPackages(rootfsPath);
      final currentSet = currentPackages.map((e) => e.trim()).toSet();
      
      final lines = worldFile.readAsLinesSync();
      final List<String> updatedLines = [];
      bool modified = false;

      for (final line in lines) {
        final trimmed = line.trim();
        if (trimmed.isEmpty) continue;

        // If the package is installed, or it's part of the current installation, keep it.
        // Otherwise, if it is NOT installed and NOT part of the current installation, it is a stale failed package. Remove it.
        if (installed.contains(trimmed) || currentSet.contains(trimmed)) {
          updatedLines.add(line);
        } else {
          modified = true;
          debugPrint("CodeDroid: Removing stale/failed package '$trimmed' from /etc/apk/world");
        }
      }

      if (modified) {
        worldFile.writeAsStringSync("${updatedLines.join('\n')}\n");
      }
    } catch (e) {
      debugPrint("CodeDroid Error sanitizing /etc/apk/world: $e");
    }
  }

  static Future<void> _makeRootfsWritable(String rootfsPath) async {
    try {
      await Process.run("chmod", ["-R", "u+w", rootfsPath]);
    } catch (e) {
      debugPrint("CodeDroid Warning: Failed to chmod u+w rootfs: $e");
    }
  }
}

class TarExtractor {
  static String pathJoin(String a, String b) {
    if (a.isEmpty) return b;
    if (b.isEmpty) return a;
    if (a.endsWith('/')) {
      return '$a$b';
    }
    return '$a/$b';
  }

  static String pathDirname(String p) {
    int idx = p.lastIndexOf('/');
    return idx == -1 ? '.' : p.substring(0, idx);
  }

  static Future<void> extract(String tarPath, String destDir) async {
    final file = File(tarPath);
    final bytes = await file.readAsBytes();
    int offset = 0;
    
    while (offset + 512 <= bytes.length) {
      final header = bytes.sublist(offset, offset + 512);
      if (header.every((b) => b == 0)) {
        break; // End of archive
      }
      
      String name = _parseString(header, 0, 100);
      final prefix = _parseString(header, 345, 155);
      if (prefix.isNotEmpty) {
        name = '$prefix/$name';
      }
      
      final size = _parseOctal(header, 124, 12);
      final typeflag = String.fromCharCode(header[156]);
      final linkname = _parseString(header, 157, 100);
      
      offset += 512;
      
      // Skip PaxHeaders and other POSIX metadata
      if (typeflag == 'x' || typeflag == 'g' || name.startsWith('PaxHeaders/') || name.contains('/PaxHeaders/')) {
        final padding = (512 - (size % 512)) % 512;
        offset += size + padding;
        continue;
      }
      
      // Skip Alpine package control metadata
      if (name.startsWith('.') || name.contains('/.')) {
        if (name == '.PKGINFO' || name.startsWith('.SIGN.')) {
          final padding = (512 - (size % 512)) % 512;
          offset += size + padding;
          continue;
        }
      }
      
      final targetPath = pathJoin(destDir, name);
      
      if (typeflag == '5') {
        // Directory
        await Directory(targetPath).create(recursive: true);
      } else if (typeflag == '0' || typeflag == '\x00' || typeflag == '1' || typeflag == '2') {
        // Pre-clean destination path to avoid write-through of symbolic/hard links
        try {
          final link = Link(targetPath);
          final file = File(targetPath);
          final dir = Directory(targetPath);
          if (FileSystemEntity.isLinkSync(targetPath)) {
            await link.delete();
          } else if (file.existsSync()) {
            await file.delete();
          } else if (dir.existsSync()) {
            await dir.delete(recursive: true);
          }
        } catch (_) {}

        if (typeflag == '0' || typeflag == '\x00') {
          // Regular file
          final fileBytes = bytes.sublist(offset, offset + size);
          await Directory(pathDirname(targetPath)).create(recursive: true);
          await File(targetPath).writeAsBytes(fileBytes);
        } else if (typeflag == '2') {
          // Symlink
          await Directory(pathDirname(targetPath)).create(recursive: true);
          final link = Link(targetPath);
          
          String resolvedTarget = linkname;
          if (resolvedTarget.startsWith('/')) {
            final linkDirGuest = pathDirname(name);
            final components = linkDirGuest.split('/');
            int levels = components.length;
            if (linkDirGuest.isEmpty || linkDirGuest == '.') levels = 0;
            
            String prefix = '';
            if (levels == 0) {
              prefix = '.';
            } else {
              prefix = List.generate(levels, (_) => '..').join('/');
            }
            final relativeTarget = resolvedTarget.substring(1);
            resolvedTarget = "$prefix/$relativeTarget";
          }
          
          await link.create(resolvedTarget);
        } else if (typeflag == '1') {
          // Hardlink - copy the target file!
          await Directory(pathDirname(targetPath)).create(recursive: true);
          final sourcePath = pathJoin(destDir, linkname);
          final sourceFile = File(sourcePath);
          if (await sourceFile.exists()) {
            await sourceFile.copy(targetPath);
          }
        }
      }
      
      final padding = (512 - (size % 512)) % 512;
      offset += size + padding;
    }
  }
  
  static String _parseString(Uint8List bytes, int offset, int length) {
    int end = offset;
    while (end < offset + length && bytes[end] != 0) {
      end++;
    }
    return utf8.decode(bytes.sublist(offset, end)).trim();
  }
  
  static int _parseOctal(Uint8List bytes, int offset, int length) {
    int value = 0;
    for (int i = 0; i < length; i++) {
      final b = bytes[offset + i];
      if (b == 0 || b == 32) {
        if (value > 0) break;
        continue;
      }
      if (b < 48 || b > 55) {
        continue;
      }
      value = (value << 3) + (b - 48);
    }
    return value;
  }
}
