import 'dart:io';
import '../../services/download_service.dart';
import '../../services/environment_service.dart';
import '../../utils/path_utils.dart';

/// Installs the Kotlin compiler (JetBrains official release) + JDK dependency.
class KotlinInstaller {
  KotlinInstaller._();

  static const String _kotlinVersion = '1.9.24';

  static Future<void> install(
    String rootfsPath,
    String tmpPath,
    void Function(String) onProgress,
  ) async {
    onProgress('Starting Kotlin installation...');

    // 1. Install JDK via APK first
    await _installJdk(rootfsPath, onProgress);

    // 2. Download Kotlin compiler zip
    final kotlinUrl =
        'https://github.com/JetBrains/kotlin/releases/download/'
        'v$_kotlinVersion/kotlin-compiler-$_kotlinVersion.zip';
    final kotlinZipPath = '$tmpPath/kotlin-compiler.zip';

    try {
      onProgress('Downloading Kotlin compiler $_kotlinVersion...');
      await DownloadService.download(kotlinUrl, kotlinZipPath, onProgress: (p, dl, total) {
        if (total != null && total > 0) {
          final mb = (dl / 1024 / 1024).toStringAsFixed(1);
          final totalMb = (total / 1024 / 1024).toStringAsFixed(1);
          onProgress('[kotlin] $mb / $totalMb MB');
        }
      });

      // 3. Extract
      onProgress('Extracting Kotlin compiler...');
      final destDir = Directory('$rootfsPath/usr/share/kotlin');
      if (destDir.existsSync()) {
        try { destDir.deleteSync(recursive: true); } catch (_) {}
      }
      destDir.createSync(recursive: true);

      final unzipResult = await Process.run('unzip', [
        '-o', kotlinZipPath,
        '-d', '$rootfsPath/usr/share/kotlin',
      ]);
      if (unzipResult.exitCode != 0) {
        throw Exception('unzip failed: ${unzipResult.stderr}');
      }

      // 4. Create symlinks
      onProgress('Creating Kotlin symlinks...');
      final binDir = Directory('$rootfsPath/usr/bin');
      if (!binDir.existsSync()) binDir.createSync(recursive: true);

      for (final bin in ['kotlin', 'kotlinc']) {
        final link = Link('$rootfsPath/usr/bin/$bin');
        if (link.existsSync()) { try { link.deleteSync(); } catch (_) {} }
        await link.create('../share/kotlin/kotlinc/bin/$bin');
      }

      try {
        await Process.run('chmod', [
          '+x',
          '$rootfsPath/usr/share/kotlin/kotlinc/bin/kotlin',
          '$rootfsPath/usr/share/kotlin/kotlinc/bin/kotlinc',
        ]);
      } catch (_) {}

      EnvironmentService.convertAbsoluteSymlinksToRelative(rootfsPath);
      onProgress('SUCCESS: Kotlin $_kotlinVersion installed!');
    } finally {
      PathUtils.deleteFileOrLink(kotlinZipPath);
    }
  }

  static Future<void> _installJdk(
    String rootfsPath,
    void Function(String) onProgress,
  ) async {
    final linuxDir = Directory(rootfsPath).parent.path;
    final prootPath = PathUtils.canonicalize('$linuxDir/proot');

    onProgress('Installing OpenJDK 17 (required for Kotlin)...');
    try {
      final result = await Process.run(
        prootPath,
        [
          '-0', '--link2symlink',
          '-r', rootfsPath,
          '-w', '/',
          '-b', '/dev', '-b', '/proc', '-b', '/sys',
          '/sbin/apk', 'add', '--no-cache', 'openjdk17', 'maven', 'gradle',
        ],
        workingDirectory: linuxDir,
        environment: EnvironmentService.buildEnvironment(
          tmpPath: '$linuxDir/tmp',
          l2sPath: '$rootfsPath/.l2s',
        ),
      );
      if (result.exitCode != 0) {
        onProgress('Warning: JDK install failed (non-fatal): ${result.stderr}');
      } else {
        onProgress('OpenJDK 17 installed.');
      }
    } catch (e) {
      onProgress('Warning: JDK install exception (non-fatal): $e');
    }
  }

  // ─── Removal ────────────────────────────────────────────────────────────────

  static Future<void> uninstall(
    String rootfsPath,
    String linuxDir,
    void Function(String) onProgress,
  ) async {
    onProgress('Removing Kotlin...');
    final kotlinDir = Directory('$rootfsPath/usr/share/kotlin');
    if (kotlinDir.existsSync()) {
      try { kotlinDir.deleteSync(recursive: true); } catch (e) {
        onProgress('Warning: $e');
      }
    }
    for (final f in ['kotlin', 'kotlinc']) {
      PathUtils.deleteFileOrLink('$rootfsPath/usr/bin/$f');
    }
    // Also remove npm kotlin-language-server if present
    try {
      final prootPath = PathUtils.canonicalize('$linuxDir/proot');
      await Process.run(
        prootPath,
        [
          '-0', '--link2symlink',
          '-r', rootfsPath, '-w', '/',
          '-b', '/dev', '-b', '/proc', '-b', '/sys',
          '/bin/sh', '-c', 'npm uninstall -g kotlin-language-server || true',
        ],
        workingDirectory: linuxDir,
        environment: EnvironmentService.buildEnvironment(
          tmpPath: '$linuxDir/tmp',
          l2sPath: '$rootfsPath/.l2s',
        ),
      );
    } catch (_) {}
    onProgress('SUCCESS: Kotlin removed.');
  }
}
