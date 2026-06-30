import 'dart:io';
import '../../services/download_service.dart';
import '../../services/environment_service.dart';
import '../../utils/tar_extractor.dart';
import '../../utils/path_utils.dart';

/// Installs Go compiler + gopls LSP manually by downloading pre-built APKs
/// from the Alpine v3.18 repository. Works for both aarch64 and x86_64.
class GoInstaller {
  GoInstaller._();

  /// Alpine v3.18 package list for Go.
  static const Map<String, List<String>> _packages = {
    'aarch64': [
      'binutils-gold-2.40-r8.apk',
      'go-1.20.11-r0.apk',
      'gopls-0.11.0-r8.apk',
    ],
    'x86_64': [
      'binutils-gold-2.40-r8.apk',
      'go-1.20.11-r0.apk',
      'gopls-0.11.0-r8.apk',
    ],
  };

  static String _baseUrl(String pkgName, String arch) {
    // go and gopls are in community; binutils-gold is in main
    if (pkgName.startsWith('go-') || pkgName.startsWith('gopls-')) {
      return 'https://dl-cdn.alpinelinux.org/alpine/v3.18/community/$arch/';
    }
    return 'https://dl-cdn.alpinelinux.org/alpine/v3.18/main/$arch/';
  }

  static Future<void> install(
    String rootfsPath,
    String tmpPath,
    void Function(String) onProgress,
  ) async {
    final arch = await EnvironmentService.getArchitecture();
    final packages = _packages[arch] ?? _packages['aarch64']!;

    onProgress('Starting manual Go installation for $arch...');

    // Pre-cleanup conflicting files
    await _cleanupConflicts(rootfsPath, onProgress);

    // Ensure shared libraries are available first
    onProgress('Installing shared library dependencies...');
    await _installDepsViaApk(rootfsPath, onProgress);

    // Clean tmp
    _cleanTmp(tmpPath);

    for (final pkg in packages) {
      final name = pkg.split('-').first;
      final apkPath = '$tmpPath/$pkg';
      final tarPath = '$tmpPath/${pkg.replaceAll('.apk', '.tar')}';

      // Check guest cache
      final cached = File('$rootfsPath/tmp/go-apks/$pkg');
      if (cached.existsSync()) {
        onProgress('[$name] Using cached package...');
        try {
          cached.copySync(apkPath);
        } catch (_) {
          onProgress('[$name] Cache copy failed, downloading fresh...');
          await _download(_baseUrl(pkg, arch) + pkg, apkPath, name, onProgress);
        }
      } else {
        await _download(_baseUrl(pkg, arch) + pkg, apkPath, name, onProgress);
      }

      try {
        onProgress('[$name] Decompressing...');
        await DownloadService.decompressGzip(apkPath, tarPath);
        onProgress('[$name] Extracting files...');
        await TarExtractor.extract(tarPath, rootfsPath);
        onProgress('[$name] Done!');
      } catch (e) {
        onProgress('ERROR: Failed to extract $name: $e');
        rethrow;
      } finally {
        PathUtils.deleteFileOrLink(apkPath);
        PathUtils.deleteFileOrLink(tarPath);
      }
    }

    // Create /usr/local/go symlink so GOROOT works
    await _setupGoroot(rootfsPath, onProgress);

    EnvironmentService.convertAbsoluteSymlinksToRelative(rootfsPath);
    onProgress('SUCCESS: Go compiler and gopls LSP installed!');
  }

  static Future<void> _download(
    String url,
    String destPath,
    String name,
    void Function(String) onProgress,
  ) async {
    onProgress('[$name] Downloading...');
    try {
      await DownloadService.download(url, destPath, onProgress: (p, dl, total) {
        if (total != null && total > 0) {
          final pct = (dl / total * 100).toStringAsFixed(0);
          onProgress('[$name] $pct%');
        }
      });
    } catch (e) {
      onProgress('ERROR: Failed to download $name: $e');
      rethrow;
    }
  }

  static Future<void> _installDepsViaApk(
    String rootfsPath,
    void Function(String) onProgress,
  ) async {
    // We'll call apk through a quick proot invocation
    final linuxDir = Directory(rootfsPath).parent.path;
    final prootPath = PathUtils.canonicalize('$linuxDir/proot');
    final tmpPath = '$linuxDir/tmp';
    final l2sPath = '$rootfsPath/.l2s';

    try {
      final result = await Process.run(
        prootPath,
        [
          '-0', '--link2symlink',
          '-r', rootfsPath,
          '-w', '/',
          '-b', '/dev', '-b', '/proc', '-b', '/sys',
          '/sbin/apk', 'add', '--no-cache', 'libgcc', 'libstdc++', 'zstd-libs', 'zlib',
        ],
        workingDirectory: linuxDir,
        environment: EnvironmentService.buildEnvironment(
          tmpPath: tmpPath,
          l2sPath: l2sPath,
        ),
      );
      if (result.exitCode != 0) {
        onProgress('Warning: dependency install failed: ${result.stderr}');
      }
    } catch (e) {
      onProgress('Warning: Could not pre-install deps: $e');
    }
  }

  static Future<void> _setupGoroot(
    String rootfsPath,
    void Function(String) onProgress,
  ) async {
    // go package installs to /usr/lib/go; create /usr/local/go symlink
    final goLibDir = Directory('$rootfsPath/usr/lib/go');
    if (goLibDir.existsSync()) {
      final symlink = Link('$rootfsPath/usr/local/go');
      if (symlink.existsSync()) {
        try { symlink.deleteSync(); } catch (_) {}
      }
      try {
        symlink.createSync('../lib/go');
        onProgress('Created /usr/local/go → ../lib/go symlink for GOROOT.');
      } catch (e) {
        onProgress('Warning: Could not create GOROOT symlink: $e');
      }
    }
  }

  static Future<void> _cleanupConflicts(
    String rootfsPath,
    void Function(String) onProgress,
  ) async {
    final arch = await EnvironmentService.getArchitecture();
    final targetTuple = '$arch-alpine-linux-musl';

    final dirsToCreate = [
      '$rootfsPath/usr/bin',
      '$rootfsPath/usr/lib/go',
      '$rootfsPath/usr/local/go',
      '$rootfsPath/usr/$targetTuple/bin',
      '$rootfsPath/.l2s/usr/bin',
      '$rootfsPath/.l2s/usr/lib/go',
      '$rootfsPath/.l2s/usr/$targetTuple/bin',
    ];
    for (final d in dirsToCreate) {
      try { Directory(d).createSync(recursive: true); } catch (_) {}
    }

    for (final rel in [
      'usr/bin/go',
      'usr/bin/gofmt',
      'usr/bin/gopls',
      'usr/bin/ld.gold',
      'usr/$targetTuple/bin/ld.gold',
    ]) {
      _deleteWithL2s(rootfsPath, rel);
    }
    onProgress('Cleaned up conflicting Go files.');
  }

  static void _deleteWithL2s(String rootfsPath, String relPath) {
    PathUtils.deleteFileOrLink('$rootfsPath/.l2s/$relPath');
    PathUtils.deleteFileOrLink('$rootfsPath/$relPath');
  }

  static void _cleanTmp(String tmpPath) {
    final tmpDir = Directory(tmpPath);
    if (!tmpDir.existsSync()) return;
    try {
      for (final f in tmpDir.listSync()) {
        try { f.deleteSync(recursive: true); } catch (_) {}
      }
    } catch (_) {}
  }

  // ─── Removal ────────────────────────────────────────────────────────────────

  static Future<void> uninstall(
    String rootfsPath,
    void Function(String) onProgress,
  ) async {
    onProgress('Removing Go installation...');
    for (final d in [
      '$rootfsPath/usr/lib/go',
      '$rootfsPath/usr/local/go',
    ]) {
      final dir = Directory(d);
      if (dir.existsSync()) {
        try { dir.deleteSync(recursive: true); onProgress('Deleted $d'); }
        catch (e) { onProgress('Warning: $e'); }
      }
    }
    for (final f in ['go', 'gofmt', 'gopls']) {
      PathUtils.deleteFileOrLink('$rootfsPath/usr/bin/$f');
    }
    onProgress('SUCCESS: Go removed.');
  }
}
