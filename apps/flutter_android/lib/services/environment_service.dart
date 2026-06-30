import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:path_provider/path_provider.dart';
import '../utils/path_utils.dart';

/// Manages the Alpine Linux rootfs environment:
///  - rootfs directory layout
///  - proot binary
///  - symlink canonicalization
///  - l2s (link2symlink) metadata mirroring
///  - environment variable map construction
class EnvironmentService {
  EnvironmentService._();

  // ─── Directory / Path helpers ───────────────────────────────────────────────

  static Future<String> getLinuxDir() async {
    final appDir = await getApplicationSupportDirectory();
    final canonical = PathUtils.canonicalize(appDir.path);
    return '$canonical/linux';
  }

  static Future<String> getRootfsPath() async {
    final linuxDir = await getLinuxDir();
    return PathUtils.canonicalize('$linuxDir/rootfs');
  }

  static Future<String> getProotPath() async {
    final linuxDir = await getLinuxDir();
    return PathUtils.canonicalize('$linuxDir/proot');
  }

  // ─── Arch detection ────────────────────────────────────────────────────────

  /// Returns `"aarch64"` (ARM64) or `"x86_64"`.
  static Future<String> getArchitecture() async {
    try {
      final result = await Process.run('getprop', ['ro.product.cpu.abi']);
      final abi = result.stdout.toString().trim();
      if (abi.contains('arm64') || abi.contains('aarch64')) return 'aarch64';
      if (abi.contains('x86_64')) return 'x86_64';
    } catch (_) {}
    return 'aarch64'; // safe default
  }

  // ─── Environment map ────────────────────────────────────────────────────────

  /// Builds a clean environment map for PRoot / guest processes.
  static Map<String, String> buildEnvironment({
    required String tmpPath,
    required String l2sPath,
    bool appendHostPath = false,
    Map<String, String>? extra,
  }) {
    final env = Map<String, String>.from(Platform.environment)
      ..remove('LD_PRELOAD')
      ..remove('LD_LIBRARY_PATH');

    const guestPath =
        '/usr/local/go/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin';

    if (appendHostPath) {
      final hostPath = Platform.environment['PATH'] ?? '';
      env['PATH'] = '$guestPath:$hostPath';
    } else {
      env['PATH'] = guestPath;
    }

    env.addAll({
      'HOME': '/root',
      'USER': 'root',
      'TERM': 'xterm-256color',
      'PROOT_TMP_DIR': tmpPath,
      'PROOT_L2S_DIR': l2sPath,
      'TMPDIR': '/tmp',
      'TMP': '/tmp',
      'TEMP': '/tmp',
      // Go env — prevents gopls from crashing when no GOROOT is set
      'GOROOT': '/usr/local/go',
      'GOPATH': '/root/go',
      'GOTMPDIR': '/root/tmp',
      'GOCACHE': '/root/.cache/go-build',
      'CGO_ENABLED': '0',
    });

    if (extra != null) env.addAll(extra);
    return env;
  }

  // ─── Rootfs validation ──────────────────────────────────────────────────────

  static bool isRootfsValid(String rootfsPath) {
    final apk = '$rootfsPath/sbin/apk';
    final sh = '$rootfsPath/bin/sh';
    return Directory(rootfsPath).existsSync() &&
        (File(apk).existsSync() || Link(apk).existsSync()) &&
        (File(sh).existsSync() || Link(sh).existsSync());
  }

  static bool isProotValid(String prootPath) {
    final f = File(prootPath);
    if (!f.existsSync() || f.lengthSync() < 1000) return false;
    try {
      final raf = f.openSync(mode: FileMode.read);
      final magic = raf.readSync(4);
      raf.closeSync();
      return magic.length == 4 &&
          magic[0] == 0x7f &&
          magic[1] == 0x45 &&
          magic[2] == 0x4c &&
          magic[3] == 0x46;
    } catch (_) {
      return false;
    }
  }

  // ─── Asset extraction ───────────────────────────────────────────────────────

  static Future<void> copyAssetBinary(String assetPath, String destPath) async {
    final data = await rootBundle.load(assetPath);
    await File(destPath).writeAsBytes(data.buffer.asUint8List());
    await Process.run('chmod', ['755', destPath]);
  }

  // ─── Symlink management ─────────────────────────────────────────────────────

  /// Converts every absolute symlink inside [rootfsPath] to a relative path.
  static void convertAbsoluteSymlinksToRelative(String rootfsPath) {
    debugPrint('EnvironmentService: Converting absolute symlinks to relative...');
    final rootfsDir = Directory(rootfsPath);
    if (!rootfsDir.existsSync()) return;

    int count = 0;
    try {
      for (final entity in rootfsDir.listSync(recursive: true, followLinks: false)) {
        if (entity is! Link) continue;
        try {
          final target = entity.targetSync();
          if (!target.startsWith('/')) continue;

          final linkPath = entity.path;
          final relativeLinkPath = linkPath.substring(rootfsPath.length);
          final levels = relativeLinkPath.split('/').length - 2;
          final upPrefix = levels <= 0 ? '.' : List.generate(levels, (_) => '..').join('/');
          final newTarget = '$upPrefix/${target.substring(1)}';

          entity.deleteSync();
          Link(linkPath).createSync(newTarget);
          count++;
        } catch (_) {}
      }
    } catch (e) {
      debugPrint('EnvironmentService: symlink conversion error: $e');
    }
    debugPrint('EnvironmentService: Converted $count absolute symlinks.');
  }

  // ─── l2s helpers ────────────────────────────────────────────────────────────

  static void mirrorDirectoriesToL2s(String rootfsPath) {
    debugPrint('EnvironmentService: Mirroring directory structure to .l2s...');
    final l2sBase = '$rootfsPath/.l2s';
    try {
      for (final entity in Directory(rootfsPath).listSync(recursive: true, followLinks: false)) {
        if (entity is! Directory) continue;
        final rel = entity.path.substring(rootfsPath.length);
        if (rel.isEmpty || rel.startsWith('/.l2s')) continue;
        final target = Directory('$l2sBase$rel');
        if (!target.existsSync()) target.createSync(recursive: true);
      }
    } catch (e) {
      debugPrint('EnvironmentService: l2s mirror error: $e');
    }
  }

  static void cleanOrphanL2s(String rootfsPath) {
    debugPrint('EnvironmentService: Cleaning orphan .l2s entries...');
    final l2sBase = '$rootfsPath/.l2s';
    final l2sDir = Directory(l2sBase);
    if (!l2sDir.existsSync()) return;

    try {
      for (final entity in l2sDir.listSync(recursive: true, followLinks: false)) {
        if (entity is! File && entity is! Link) continue;
        final rel = entity.path.substring(l2sBase.length);
        if (rel.isEmpty) continue;
        final root = '$rootfsPath$rel';
        if (!File(root).existsSync() &&
            !Link(root).existsSync() &&
            !Directory(root).existsSync()) {
          try {
            entity.deleteSync();
          } catch (_) {}
        }
      }
    } catch (e) {
      debugPrint('EnvironmentService: l2s clean error: $e');
    }
  }

  // ─── Misc helpers ───────────────────────────────────────────────────────────

  static Future<void> makeWritable(String path) async {
    try {
      await Process.run('chmod', ['-R', 'u+rwx', path]);
    } catch (e) {
      debugPrint('EnvironmentService: chmod failed for $path: $e');
    }
  }

  static void clearStaleLock(String rootfsPath) {
    final lock = File('$rootfsPath/lib/apk/db/lock');
    if (lock.existsSync()) {
      try {
        lock.deleteSync();
        debugPrint('EnvironmentService: Cleared stale apk lock.');
      } catch (e) {
        debugPrint('EnvironmentService: Failed to clear apk lock: $e');
      }
    }
  }

  static void ensureGuestDirectories(String rootfsPath) {
    for (final path in [
      '$rootfsPath/tmp',
      '$rootfsPath/var/tmp',
      '$rootfsPath/root/tmp',
      '$rootfsPath/root/go',
      '$rootfsPath/root/.cache/go-build',
    ]) {
      try {
        Directory(path).createSync(recursive: true);
      } catch (_) {}
    }
  }

  static void writeResolvConf(String rootfsPath) {
    try {
      File('$rootfsPath/etc/resolv.conf')
        ..createSync(recursive: true)
        ..writeAsStringSync('nameserver 8.8.8.8\nnameserver 1.1.1.1\n');
    } catch (_) {}
  }

  static void writeNodeNetworkBypass(String rootfsPath) {
    final file = File('$rootfsPath/usr/local/lib/node_network_bypass.js');
    try {
      file.parent.createSync(recursive: true);
      file.writeAsStringSync('''
try {
  const os = require('os');
  if (os && typeof os.networkInterfaces === 'function') {
    os.networkInterfaces = () => ({
      lo: [{ address: '127.0.0.1', netmask: '255.0.0.0', family: 'IPv4',
             mac: '00:00:00:00:00:00', internal: true, cidr: '127.0.0.1/8' }]
    });
  }
} catch (e) { /* ignore */ }
''');
    } catch (_) {}
  }
}
