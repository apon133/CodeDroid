import 'dart:io';
import '../../services/download_service.dart';
import '../../services/environment_service.dart';
import '../../utils/tar_extractor.dart';
import '../../utils/path_utils.dart';

/// Installs build-base (GCC, G++, Make, binutils) manually from Alpine APKs.
class BuildBaseInstaller {
  BuildBaseInstaller._();

  /// Package list for Alpine v3.18 — same for aarch64 and x86_64.
  static const List<String> _packages = [
    'zstd-libs-1.5.5-r4.apk',
    'musl-1.2.4-r3.apk',
    'libgcc-12.2.1_git20220924-r10.apk',
    'libstdc++-12.2.1_git20220924-r10.apk',
    'libgomp-12.2.1_git20220924-r10.apk',
    'libatomic-12.2.1_git20220924-r10.apk',
    'gmp-6.2.1-r3.apk',
    'mpfr4-4.2.0_p12-r0.apk',
    'mpc1-1.3.1-r1.apk',
    'binutils-2.40-r8.apk',
    'gcc-12.2.1_git20220924-r10.apk',
    'musl-dev-1.2.4-r3.apk',
    'libc-dev-0.7.2-r5.apk',
    'g++-12.2.1_git20220924-r10.apk',
    'make-4.4.1-r1.apk',
    'fortify-headers-1.1-r3.apk',
  ];

  static Future<void> install(
    String rootfsPath,
    String tmpPath,
    void Function(String) onProgress,
  ) async {
    final arch = await EnvironmentService.getArchitecture();
    final baseUrl = 'https://dl-cdn.alpinelinux.org/alpine/v3.18/main/$arch/';

    onProgress('Starting build-base installation (arch: $arch)...');

    // Pre-create directories to avoid PRoot rename errors
    await _precreateDirectories(rootfsPath, arch, onProgress);

    // Clean tmp
    _cleanTmp(tmpPath);

    for (final pkg in _packages) {
      final name = pkg.split('-').first;
      final apkPath = '$tmpPath/$pkg';
      final tarPath = '$tmpPath/${pkg.replaceAll('.apk', '.tar')}';

      try {
        onProgress('[$name] Downloading...');
        await DownloadService.download('$baseUrl$pkg', apkPath, onProgress: (p, dl, total) {
          if (total != null && total > 0) {
            onProgress('[$name] ${(p * 100).toStringAsFixed(0)}%');
          }
        });

        onProgress('[$name] Decompressing...');
        await DownloadService.decompressGzip(apkPath, tarPath);

        onProgress('[$name] Extracting...');
        await TarExtractor.extract(tarPath, rootfsPath);

        onProgress('[$name] Done!');
      } catch (e) {
        onProgress('ERROR: Failed to install $name: $e');
        rethrow;
      } finally {
        PathUtils.deleteFileOrLink(apkPath);
        PathUtils.deleteFileOrLink(tarPath);
      }
    }

    EnvironmentService.convertAbsoluteSymlinksToRelative(rootfsPath);
    onProgress('SUCCESS: build-base (GCC/G++/Make) installed!');
  }

  static Future<void> _precreateDirectories(
    String rootfsPath,
    String arch,
    void Function(String) onProgress,
  ) async {
    final targetTuple = '$arch-alpine-linux-musl';
    final dirs = [
      '$rootfsPath/usr/bin',
      '$rootfsPath/usr/lib',
      '$rootfsPath/usr/libexec',
      '$rootfsPath/usr/$targetTuple/bin',
      '$rootfsPath/usr/$targetTuple/lib',
      '$rootfsPath/usr/lib/gcc/$targetTuple',
      '$rootfsPath/usr/lib/gcc/$targetTuple/12.2.1',
      '$rootfsPath/.l2s/usr/bin',
      '$rootfsPath/.l2s/usr/lib',
      '$rootfsPath/.l2s/usr/libexec',
      '$rootfsPath/.l2s/usr/$targetTuple/bin',
      '$rootfsPath/.l2s/usr/$targetTuple/lib',
      '$rootfsPath/.l2s/usr/lib/gcc/$targetTuple',
      '$rootfsPath/.l2s/usr/lib/gcc/$targetTuple/12.2.1',
    ];
    for (final d in dirs) {
      try { Directory(d).createSync(recursive: true); } catch (_) {}
    }

    // Delete any conflicting files
    final conflicting = [
      'usr/bin/ar', 'usr/bin/as', 'usr/bin/ld', 'usr/bin/ld.bfd',
      'usr/bin/nm', 'usr/bin/objcopy', 'usr/bin/objdump', 'usr/bin/ranlib',
      'usr/bin/readelf', 'usr/bin/strip', 'usr/bin/gcc', 'usr/bin/g++',
      'usr/bin/c++', 'usr/bin/cpp', 'usr/bin/gcc-ar', 'usr/bin/gcc-nm',
      'usr/bin/gcc-ranlib', 'usr/bin/make',
    ];
    for (final rel in conflicting) {
      PathUtils.deleteFileOrLink('$rootfsPath/$rel');
      PathUtils.deleteFileOrLink('$rootfsPath/.l2s/$rel');
    }
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

  static void uninstall(String rootfsPath, void Function(String) onProgress) {
    onProgress('Removing C/C++ build tools...');
    for (final f in ['gcc', 'g++', 'cpp', 'cc', 'c++', 'make', 'clangd']) {
      PathUtils.deleteFileOrLink('$rootfsPath/usr/bin/$f');
    }
    onProgress('SUCCESS: C/C++ tools removed.');
  }
}
