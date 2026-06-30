import 'dart:io';
import '../../services/download_service.dart';
import '../../services/environment_service.dart';
import '../../utils/path_utils.dart';

/// Installs the Dart SDK from the official Google storage bucket.
/// Correctly selects the arm64 or x64 binary based on device architecture.
class DartInstaller {
  DartInstaller._();

  static const String _dartVersion = '3.4.3';

  static String _dartUrl(String arch) {
    // FIX: was previously hardcoded to arm64 only
    final platformArch = arch == 'x86_64' ? 'x64' : 'arm64';
    return 'https://storage.googleapis.com/dart-archive/channels/stable/release/'
        '$_dartVersion/sdk/dartsdk-linux-$platformArch-release.zip';
  }

  static Future<void> install(
    String rootfsPath,
    String tmpPath,
    void Function(String) onProgress,
  ) async {
    final arch = await EnvironmentService.getArchitecture();
    final dartUrl = _dartUrl(arch);
    final dartZipPath = '$tmpPath/dartsdk.zip';

    onProgress('Starting Dart SDK installation (arch: $arch)...');

    // Install gcompat first (Dart uses glibc)
    await _installGcompat(rootfsPath, onProgress);

    try {
      onProgress('Downloading Dart SDK ($_dartVersion for $arch)...');
      await DownloadService.download(dartUrl, dartZipPath, onProgress: (p, dl, total) {
        if (total != null && total > 0) {
          final mb = (dl / 1024 / 1024).toStringAsFixed(1);
          final totalMb = (total / 1024 / 1024).toStringAsFixed(1);
          onProgress('[dart] $mb / $totalMb MB');
        }
      });

      onProgress('Extracting Dart SDK...');
      final dartDestDir = Directory('$rootfsPath/usr/lib/dart');
      if (dartDestDir.existsSync()) {
        try { dartDestDir.deleteSync(recursive: true); } catch (_) {}
      }
      dartDestDir.createSync(recursive: true);

      final unzipResult = await Process.run('unzip', [
        '-o',
        dartZipPath,
        '-d',
        '$rootfsPath/usr/lib/dart',
      ]);

      if (unzipResult.exitCode != 0) {
        throw Exception('unzip failed: ${unzipResult.stderr}');
      }

      onProgress('Creating Dart binary symlink...');
      final binDir = Directory('$rootfsPath/usr/bin');
      if (!binDir.existsSync()) binDir.createSync(recursive: true);

      final dartLink = Link('$rootfsPath/usr/bin/dart');
      if (dartLink.existsSync()) {
        try { dartLink.deleteSync(); } catch (_) {}
      }
      await dartLink.create('../lib/dart/dart-sdk/bin/dart');

      try {
        await Process.run('chmod', ['+x', '$rootfsPath/usr/lib/dart/dart-sdk/bin/dart']);
      } catch (_) {}

      EnvironmentService.convertAbsoluteSymlinksToRelative(rootfsPath);
      onProgress('SUCCESS: Dart SDK $_dartVersion installed!');
    } finally {
      PathUtils.deleteFileOrLink(dartZipPath);
    }
  }

  static Future<void> _installGcompat(
    String rootfsPath,
    void Function(String) onProgress,
  ) async {
    final linuxDir = Directory(rootfsPath).parent.path;
    final prootPath = PathUtils.canonicalize('$linuxDir/proot');

    try {
      onProgress('Installing glibc compatibility layer (gcompat)...');
      final result = await Process.run(
        prootPath,
        [
          '-0', '--link2symlink',
          '-r', rootfsPath,
          '-w', '/',
          '-b', '/dev', '-b', '/proc', '-b', '/sys',
          '/sbin/apk', 'add', '--no-cache', 'gcompat',
        ],
        workingDirectory: linuxDir,
        environment: EnvironmentService.buildEnvironment(
          tmpPath: '$linuxDir/tmp',
          l2sPath: '$rootfsPath/.l2s',
        ),
      );
      if (result.exitCode != 0) {
        onProgress('Warning: gcompat install failed (non-fatal): ${result.stderr}');
      } else {
        onProgress('gcompat installed successfully.');
      }
    } catch (e) {
      onProgress('Warning: gcompat install exception (non-fatal): $e');
    }
  }

  // ─── Removal ────────────────────────────────────────────────────────────────

  static void uninstall(String rootfsPath, void Function(String) onProgress) {
    onProgress('Removing Dart SDK...');
    final dartDir = Directory('$rootfsPath/usr/lib/dart');
    if (dartDir.existsSync()) {
      try { dartDir.deleteSync(recursive: true); onProgress('Deleted /usr/lib/dart'); }
      catch (e) { onProgress('Warning: $e'); }
    }
    PathUtils.deleteFileOrLink('$rootfsPath/usr/bin/dart');
    onProgress('SUCCESS: Dart removed.');
  }
}
