import 'dart:io';
import 'package:flutter/foundation.dart';
import '../../utils/path_utils.dart';
import 'environment_service.dart';
import 'installers/build_base_installer.dart';
import 'installers/dart_installer.dart';
import 'installers/go_installer.dart';
import 'installers/kotlin_installer.dart';

/// Installs and removes language packages in the Alpine rootfs.
///
/// Install flow:
///  1. Special installers handle Go, Dart, Kotlin, build-base.
///  2. Everything else goes through `apk add` via PRoot.
///
/// Every package gets the full LSP tooling installed alongside the runtime.
class PackageManager {
  PackageManager._();

  // ─── APK custom commands ────────────────────────────────────────────────────
  // Maps package key → shell command to run inside PRoot guest.
  static const Map<String, String> _apkCommands = {
    'python3':
        'apk add --no-cache python3 py3-pip && '
        'pip3 install python-lsp-server black --break-system-packages',
    'nodejs npm':
        'apk add --no-cache nodejs npm && '
        'npm install -g typescript typescript-language-server vscode-langservers-extracted',
    'rust cargo':
        'apk add --no-cache rust cargo rust-analyzer',
    'openjdk17':
        'apk add --no-cache openjdk17 maven gradle && '
        'npm install -g eclipse-jdt-ls || true',
    'csharp':
        'apk add --no-cache dotnet8-sdk || apk add --no-cache dotnet7-sdk || '
        'apk add --no-cache dotnet-sdk-6.0 || apk add --no-cache dotnet-sdk',
    'swift':     'apk add --no-cache swift',
    'ruby':      'apk add --no-cache ruby ruby-dev build-base && gem install solargraph || true',
    'javascript':
        'apk add --no-cache nodejs npm && '
        'npm install -g typescript typescript-language-server vscode-langservers-extracted',
    'typescript':
        'apk add --no-cache nodejs npm && '
        'npm install -g typescript typescript-language-server vscode-langservers-extracted',
    'vanilla-js':
        'apk add --no-cache nodejs npm && '
        'npm install -g typescript typescript-language-server vscode-langservers-extracted',
    'react':
        'apk add --no-cache nodejs npm && '
        'npm install -g typescript typescript-language-server vscode-langservers-extracted '
        '@tailwindcss/language-server',
    'vue':
        'apk add --no-cache nodejs npm && '
        'npm install -g @vue/language-server typescript',
    'svelte':
        'apk add --no-cache nodejs npm && '
        'npm install -g svelte-language-server typescript',
    'angular':
        'apk add --no-cache nodejs npm && '
        'npm install -g @angular/language-server typescript',
    'nextjs':
        'apk add --no-cache nodejs npm && '
        'npm install -g typescript typescript-language-server vscode-langservers-extracted '
        '@tailwindcss/language-server',
    'remix':
        'apk add --no-cache nodejs npm && '
        'npm install -g typescript typescript-language-server vscode-langservers-extracted',
    'git':       'apk add --no-cache git',
  };

  // ─── Install ────────────────────────────────────────────────────────────────

  static Future<void> install(
    String packageName,
    void Function(String) onProgress,
  ) async {
    final rootfsPath = await EnvironmentService.getRootfsPath();
    final linuxDir = Directory(rootfsPath).parent.path;
    final tmpPath = '$linuxDir/tmp';

    // Ensure tmp and l2s
    Directory(tmpPath).createSync(recursive: true);
    final l2sDir = Directory('$rootfsPath/.l2s');
    if (!l2sDir.existsSync()) l2sDir.createSync(recursive: true);

    await EnvironmentService.makeWritable(rootfsPath);
    EnvironmentService.mirrorDirectoriesToL2s(rootfsPath);
    EnvironmentService.cleanOrphanL2s(rootfsPath);
    EnvironmentService.clearStaleLock(rootfsPath);
    EnvironmentService.ensureGuestDirectories(rootfsPath);

    _cleanStaleApkTemp(rootfsPath);

    final trimmed = packageName.trim();
    onProgress('Starting installation: $trimmed...');

    // ── Special installers ──
    if (trimmed == 'go' || trimmed.split(' ').contains('go')) {
      await GoInstaller.install(rootfsPath, tmpPath, onProgress);
      return;
    }

    if (trimmed == 'dart' || trimmed.split(' ').contains('dart')) {
      await DartInstaller.install(rootfsPath, tmpPath, onProgress);
      return;
    }

    if (trimmed == 'kotlin' || trimmed.split(' ').contains('kotlin')) {
      await KotlinInstaller.install(rootfsPath, tmpPath, onProgress);
      return;
    }

    if (['gcc', 'g++', 'build-base'].contains(trimmed) ||
        trimmed.split(' ').contains('build-base')) {
      await BuildBaseInstaller.install(rootfsPath, tmpPath, onProgress);
      // Also install clangd for C/C++ LSP
      try {
        onProgress('Installing clangd LSP support...');
        await _runApk(rootfsPath, linuxDir, 'apk add --no-cache clang-extra-tools', onProgress);
      } catch (e) {
        onProgress('Warning: clangd install failed (non-fatal): $e');
      }
      return;
    }

    // ── Generic APK install ──
    final shellCmd = _apkCommands[trimmed];
    if (shellCmd != null) {
      await _runApk(rootfsPath, linuxDir, shellCmd, onProgress);
    } else {
      // Fallback: raw apk add
      final pkgArgs = packageName.split(' ').join(' ');
      await _runApk(
        rootfsPath,
        linuxDir,
        'apk add --no-cache --force-overwrite $pkgArgs',
        onProgress,
      );
    }

    EnvironmentService.convertAbsoluteSymlinksToRelative(rootfsPath);
    onProgress('SUCCESS: $packageName installed!');
  }

  // ─── Uninstall ──────────────────────────────────────────────────────────────

  static Future<void> uninstall(
    String packageName,
    void Function(String) onProgress,
  ) async {
    final rootfsPath = await EnvironmentService.getRootfsPath();
    final linuxDir = Directory(rootfsPath).parent.path;

    await EnvironmentService.makeWritable(rootfsPath);
    EnvironmentService.clearStaleLock(rootfsPath);

    final trimmed = packageName.trim();
    onProgress('Starting removal: $trimmed...');

    if (trimmed == 'go' || trimmed.split(' ').contains('go')) {
      await GoInstaller.uninstall(rootfsPath, onProgress);
      return;
    }

    if (trimmed == 'dart' || trimmed.split(' ').contains('dart')) {
      DartInstaller.uninstall(rootfsPath, onProgress);
      return;
    }

    if (trimmed == 'kotlin' || trimmed.split(' ').contains('kotlin')) {
      await KotlinInstaller.uninstall(rootfsPath, linuxDir, onProgress);
      return;
    }

    if (['gcc', 'g++', 'build-base'].contains(trimmed) ||
        trimmed.split(' ').contains('build-base')) {
      BuildBaseInstaller.uninstall(rootfsPath, onProgress);
      try {
        await _runApk(rootfsPath, linuxDir, 'apk del clang-extra-tools || true', onProgress);
      } catch (_) {}
      return;
    }

    // Map friendly names to actual APK package names
    String pkgs = trimmed;
    switch (trimmed) {
      case 'javascript':
      case 'typescript':
      case 'vanilla-js':
      case 'react':
      case 'vue':
      case 'svelte':
      case 'angular':
      case 'nextjs':
      case 'remix':
        pkgs = 'nodejs npm';
        break;
      case 'nodejs npm':
        pkgs = 'nodejs npm';
        break;
      case 'python3':
        pkgs = 'python3 py3-pip';
        break;
      case 'rust cargo':
        pkgs = 'rust cargo rust-analyzer';
        break;
      case 'openjdk17':
        pkgs = 'openjdk17 maven gradle';
        break;
      case 'csharp':
        pkgs = 'dotnet8-sdk dotnet7-sdk dotnet-sdk-6.0 dotnet-sdk';
        break;
    }

    try {
      await _runApk(rootfsPath, linuxDir, 'apk del --no-cache $pkgs || true', onProgress);
      onProgress('SUCCESS: $packageName removed!');
    } catch (e) {
      onProgress('ERROR: Failed to remove $packageName: $e');
    }
  }

  // ─── Run guest command ───────────────────────────────────────────────────────

  static Future<void> runGuestCommand(
    List<String> command,
    void Function(String) onProgress,
  ) async {
    final rootfsPath = await EnvironmentService.getRootfsPath();
    final linuxDir = Directory(rootfsPath).parent.path;
    final prootPath = PathUtils.canonicalize('$linuxDir/proot');
    final tmpPath = '$linuxDir/tmp';
    final l2sPath = '$rootfsPath/.l2s';

    final env = EnvironmentService.buildEnvironment(
      tmpPath: tmpPath,
      l2sPath: l2sPath,
      appendHostPath: true,
    );

    final process = await Process.start(
      prootPath,
      [
        '-0', '--link2symlink',
        '-r', rootfsPath,
        '-w', '/',
        '-b', '/dev', '-b', '/proc', '-b', '/sys',
        ...command,
      ],
      workingDirectory: linuxDir,
      environment: env,
    );

    process.stdout.listen((data) => onProgress(String.fromCharCodes(data)));
    process.stderr.listen((data) => onProgress(String.fromCharCodes(data)));

    final exitCode = await process.exitCode;
    if (exitCode != 0) {
      throw Exception('Guest command failed with exit code $exitCode');
    }
  }

  // ─── APK helper ─────────────────────────────────────────────────────────────

  static Future<void> _runApk(
    String rootfsPath,
    String linuxDir,
    String shellCmd,
    void Function(String) onProgress,
  ) async {
    final prootPath = PathUtils.canonicalize('$linuxDir/proot');
    final tmpPath = '$linuxDir/tmp';
    final l2sPath = '$rootfsPath/.l2s';

    final env = EnvironmentService.buildEnvironment(
      tmpPath: tmpPath,
      l2sPath: l2sPath,
    );

    final process = await Process.start(
      prootPath,
      [
        '-0', '--link2symlink',
        '-r', rootfsPath,
        '-w', '/',
        '-b', '/dev', '-b', '/proc', '-b', '/sys',
        '/bin/sh', '-c', shellCmd,
      ],
      workingDirectory: linuxDir,
      environment: env,
    );

    process.stdout.listen((data) => onProgress(String.fromCharCodes(data)));
    process.stderr.listen((data) => onProgress(String.fromCharCodes(data)));

    final exitCode = await process.exitCode;
    if (exitCode != 0) {
      debugPrint('PackageManager: apk command exited $exitCode for: $shellCmd');
    }
  }

  // ─── Stale temp cleanup ──────────────────────────────────────────────────────

  static void _cleanStaleApkTemp(String rootfsPath) {
    for (final scanPath in [rootfsPath, '$rootfsPath/.l2s']) {
      final dir = Directory(scanPath);
      if (!dir.existsSync()) continue;
      try {
        for (final entity in dir.listSync(recursive: true, followLinks: false)) {
          if (!entity.path.split('/').last.contains('.apk.')) continue;
          try {
            if (entity is Link) {
              final target = entity.targetSync();
              if (target.contains('.l2s')) {
                final backingPath =
                    Uri.file(entity.path).resolve(target).toFilePath();
                PathUtils.deleteFileOrLink(backingPath);
              }
            }
            entity.deleteSync();
          } catch (_) {}
        }
      } catch (_) {}
    }
  }

  // ─── APK world sanitizer ────────────────────────────────────────────────────

  static Set<String> getInstalledApkPackages(String rootfsPath) {
    final installed = <String>{};
    final dbFile = File('$rootfsPath/lib/apk/db/installed');
    if (!dbFile.existsSync()) return installed;
    try {
      for (final line in dbFile.readAsLinesSync()) {
        if (line.startsWith('P:')) installed.add(line.substring(2).trim());
      }
    } catch (_) {}
    return installed;
  }
}
