import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'services/environment_service.dart';
import 'services/process_manager.dart';
import 'services/package_manager.dart';
import 'utils/path_utils.dart';

/// Public facade for the CodeDroid Linux subsystem.
///
/// All implementation details live in focused service classes:
///  - [EnvironmentService]  — rootfs setup, symlinks, env vars
///  - [ProcessManager]      — API process lifecycle + auto-restart
///  - [PackageManager]      — install / remove language packages
///  - [DownloadService]     — HTTP downloads with retry & timeout
///
/// External code (main.dart, language_setup_sheet.dart, terminal_screen.dart)
/// only needs to import this file.
class LinuxManager {
  LinuxManager._();

  // ─── Boot ────────────────────────────────────────────────────────────────────

  /// Full initialization: rootfs setup → proot → API start.
  static Future<void> initialize() async {
    debugPrint('LinuxManager: initialize() started.');
    try {
      await ProcessManager.killStale();

      final arch = await EnvironmentService.getArchitecture();
      debugPrint('LinuxManager: Architecture = $arch');

      final linuxDir = await EnvironmentService.getLinuxDir();
      final linuxDirObj = Directory(linuxDir);
      if (!linuxDirObj.existsSync()) linuxDirObj.createSync(recursive: true);
      final linuxDirResolved = linuxDirObj.resolveSymbolicLinksSync();

      final rootfsDirPath = '$linuxDirResolved/rootfs';
      final prootFilePath = '$linuxDirResolved/proot';
      final alpineTarPath = '$linuxDirResolved/alpine-minirootfs.tar.gz';

      final rootfsDir = Directory(rootfsDirPath);
      final prootFile = File(prootFilePath);

      // Validate existing installation
      final isRootfsValid = EnvironmentService.isRootfsValid(rootfsDirPath);
      final isProotValid = EnvironmentService.isProotValid(prootFilePath);

      debugPrint('LinuxManager: isRootfsValid=$isRootfsValid, isProotValid=$isProotValid');

      // Clean up if invalid
      if (!isRootfsValid && linuxDirObj.existsSync()) {
        debugPrint('LinuxManager: Rootfs invalid — clearing for re-extraction.');
        try { Directory(linuxDirResolved).deleteSync(recursive: true); } catch (e) {
          debugPrint('LinuxManager: Failed to clear linuxDir: $e');
        }
        Directory(linuxDirResolved).createSync(recursive: true);
      } else if (!isProotValid && prootFile.existsSync()) {
        debugPrint('LinuxManager: PRoot binary corrupt — re-copying.');
        try { prootFile.deleteSync(); } catch (_) {}
      }

      // Extract assets if needed
      if (!rootfsDir.existsSync() || !prootFile.existsSync()) {
        debugPrint('LinuxManager: Initializing Linux environment from assets...');
        await _extractAssets(arch, linuxDirResolved, prootFilePath, alpineTarPath, rootfsDirPath);
      }

      // Resolve canonical rootfs path
      final rootfsResolved = Directory(rootfsDirPath).resolveSymbolicLinksSync();
      debugPrint('LinuxManager: rootfsPath=$rootfsResolved');

      // Post-extraction setup
      EnvironmentService.writeResolvConf(rootfsResolved);
      EnvironmentService.ensureGuestDirectories(rootfsResolved);
      EnvironmentService.writeNodeNetworkBypass(rootfsResolved);

      // Copy / update API binary
      await _updateApiBinary(arch, rootfsResolved);

      // Fix permissions and l2s
      await EnvironmentService.makeWritable(rootfsResolved);
      final l2sDir = Directory('$rootfsResolved/.l2s');
      if (!l2sDir.existsSync()) l2sDir.createSync(recursive: true);
      EnvironmentService.mirrorDirectoriesToL2s(rootfsResolved);
      EnvironmentService.cleanOrphanL2s(rootfsResolved);
      await EnvironmentService.makeWritable(l2sDir.path);

      // Convert absolute symlinks to relative (PRoot requirement)
      EnvironmentService.convertAbsoluteSymlinksToRelative(rootfsResolved);

      // Clear stale APK lock
      EnvironmentService.clearStaleLock(rootfsResolved);

      // Launch API
      await ProcessManager.startApiServer(prootFilePath, rootfsResolved);
    } catch (e, st) {
      debugPrint('LinuxManager: initialization error: $e\n$st');
    }
  }

  // ─── Public API ──────────────────────────────────────────────────────────────

  /// Install a language package (e.g. "go", "dart", "python3").
  static Future<void> runApkAdd(
    String packageName,
    void Function(String) onProgress,
  ) =>
      PackageManager.install(packageName, onProgress);

  /// Remove a language package.
  static Future<void> deletePackage(
    String packageName,
    void Function(String) onProgress,
  ) =>
      PackageManager.uninstall(packageName, onProgress);

  /// Run an arbitrary command inside the PRoot guest.
  static Future<void> runGuestCommand(
    List<String> command,
    void Function(String) onProgress,
  ) =>
      PackageManager.runGuestCommand(command, onProgress);

  /// Stop the API process (e.g. on app exit).
  static Future<void> stop() => ProcessManager.stop();

  /// Logs from the API process.
  static List<String> get processLogs => ProcessManager.logs;

  /// Backwards-compat helpers used by terminal / log screens.
  static String canonicalizePath(String path) => PathUtils.canonicalize(path);

  // ─── Private helpers ─────────────────────────────────────────────────────────

  static Future<void> _extractAssets(
    String arch,
    String linuxDir,
    String prootPath,
    String alpineTarPath,
    String rootfsDirPath,
  ) async {
    // 1. Copy proot binary
    debugPrint('LinuxManager: Copying proot from assets...');
    await EnvironmentService.copyAssetBinary(
      'assets/linux/$arch/proot',
      prootPath,
    );

    // 2. Copy Alpine tarball
    debugPrint('LinuxManager: Copying Alpine rootfs tarball from assets...');
    final ByteData alpineData = await rootBundle.load('assets/linux/$arch/alpine-minirootfs.tar.gz');
    await File(alpineTarPath).writeAsBytes(alpineData.buffer.asUint8List());

    // 3. Extract tarball
    Directory(rootfsDirPath).createSync(recursive: true);
    debugPrint('LinuxManager: Extracting Alpine tarball...');
    final tarResult = await Process.run('tar', [
      '-xzf', alpineTarPath,
      '-C', rootfsDirPath,
    ]);
    if (tarResult.exitCode != 0) {
      throw Exception('tar extraction failed: ${tarResult.stderr}');
    }

    // 4. Cleanup tarball
    try { File(alpineTarPath).deleteSync(); } catch (_) {}
    debugPrint('LinuxManager: Asset extraction complete.');
  }

  static Future<void> _updateApiBinary(String arch, String rootfsPath) async {
    final apiFile = File('$rootfsPath/usr/local/bin/codedroid_api');
    try {
      apiFile.parent.createSync(recursive: true);
      debugPrint('LinuxManager: Copying codedroid_api binary...');
      final ByteData apiData = await rootBundle.load('assets/linux/$arch/codedroid_api');
      await apiFile.writeAsBytes(apiData.buffer.asUint8List());
      await Process.run('chmod', ['755', apiFile.path]);
      debugPrint('LinuxManager: API binary updated.');
    } catch (e) {
      debugPrint('LinuxManager: No bundled codedroid_api found (skipping): $e');
    }
  }
}
