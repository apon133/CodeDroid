import 'dart:async';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'environment_service.dart';

/// Manages the lifecycle of the CodeDroid API backend process:
///  - Start inside PRoot
///  - Capture stdout / stderr logs
///  - Auto-restart on unexpected exit (up to [_maxRestarts] times)
///  - Graceful shutdown
class ProcessManager {
  ProcessManager._();

  static Process? _apiProcess;
  static String? _prootPath;
  static String? _rootfsPath;

  static final List<String> logs = [];
  static const int _maxLogs = 3000;
  static const int _maxRestarts = 5;
  static int _restartCount = 0;
  static bool _intentionallyStopped = false;

  /// Starts the API server inside PRoot.
  static Future<void> startApiServer(String prootPath, String rootfsPath) async {
    if (_apiProcess != null) {
      debugPrint('ProcessManager: API already running (PID ${_apiProcess!.pid}). Skipping.');
      return;
    }

    _prootPath = prootPath;
    _rootfsPath = rootfsPath;
    _intentionallyStopped = false;

    await _launch(prootPath, rootfsPath);
  }

  static Future<void> _launch(String prootPath, String rootfsPath) async {
    debugPrint('ProcessManager: Launching API server...');

    final linuxDir = Directory(rootfsPath).parent.path;

    final tmpDir = Directory('$linuxDir/tmp');
    if (!tmpDir.existsSync()) tmpDir.createSync(recursive: true);
    final tmpPath = tmpDir.resolveSymbolicLinksSync();

    final l2sDir = Directory('$rootfsPath/.l2s');
    if (!l2sDir.existsSync()) l2sDir.createSync(recursive: true);
    final l2sPath = l2sDir.resolveSymbolicLinksSync();

    final env = EnvironmentService.buildEnvironment(
      tmpPath: tmpPath,
      l2sPath: l2sPath,
      appendHostPath: true,
      extra: {
        'NODE_OPTIONS': '--require /usr/local/lib/node_network_bypass.js',
      },
    );

    final args = [
      '-0',
      '--link2symlink',
      '-r', rootfsPath,
      '-w', '/',
      '-b', '/dev',
      '-b', '/proc',
      '-b', '/sys',
      '/usr/local/bin/codedroid_api',
    ];

    try {
      _apiProcess = await Process.start(
        prootPath,
        args,
        workingDirectory: linuxDir,
        environment: env,
      );

      debugPrint('ProcessManager: API started — PID ${_apiProcess!.pid}');
      _addLog('[API STARTED] PID: ${_apiProcess!.pid}');

      _apiProcess!.stdout.listen(
        (data) => _handleOutput('[STDOUT]', data),
        onError: (e) => _addLog('[STDOUT ERROR] $e'),
        onDone: () => _addLog('[STDOUT] Stream closed.'),
      );

      _apiProcess!.stderr.listen(
        (data) => _handleOutput('[STDERR]', data),
        onError: (e) => _addLog('[STDERR ERROR] $e'),
        onDone: () => _addLog('[STDERR] Stream closed.'),
      );

      _apiProcess!.exitCode.then(_onExit);
    } catch (e, st) {
      debugPrint('ProcessManager: Failed to launch API: $e\n$st');
      _addLog('[API LAUNCH ERROR] $e');
      _apiProcess = null;
    }
  }

  static void _handleOutput(String prefix, List<int> data) {
    final text = String.fromCharCodes(data);
    debugPrint('$prefix $text');
    for (final line in text.split('\n')) {
      if (line.trim().isNotEmpty) _addLog('$prefix ${line.trim()}');
    }
  }

  static void _addLog(String entry) {
    logs.add(entry);
    if (logs.length > _maxLogs) {
      logs.removeRange(0, logs.length - _maxLogs);
    }
  }

  static void _onExit(int code) {
    debugPrint('ProcessManager: API exited with code $code');
    _addLog('[API EXITED] Exit code: $code');
    _apiProcess = null;

    if (_intentionallyStopped) {
      debugPrint('ProcessManager: Intentional stop — no restart.');
      return;
    }

    if (_restartCount >= _maxRestarts) {
      _addLog('[API] Max restarts reached ($_maxRestarts). Not restarting.');
      debugPrint('ProcessManager: Max restarts reached. Giving up.');
      return;
    }

    _restartCount++;
    final delay = Duration(seconds: _restartCount * 2); // 2s, 4s, 6s …
    _addLog('[API] Restarting in ${delay.inSeconds}s (attempt $_restartCount/$_maxRestarts)...');
    debugPrint('ProcessManager: Restarting in ${delay.inSeconds}s (attempt $_restartCount)...');

    Future.delayed(delay, () {
      if (!_intentionallyStopped && _apiProcess == null) {
        _launch(_prootPath!, _rootfsPath!);
      }
    });
  }

  /// Gracefully kills the API process.
  static Future<void> stop() async {
    _intentionallyStopped = true;
    _restartCount = 0;
    if (_apiProcess != null) {
      _apiProcess!.kill();
      _apiProcess = null;
      _addLog('[API STOPPED] Killed by user/system.');
    }
  }

  /// Kills any stale proot / codedroid_api processes left from a previous session.
  static Future<void> killStale() async {
    debugPrint('ProcessManager: Killing stale background processes...');
    try {
      await Process.run('pkill', ['-f', 'codedroid_api']);
      await Process.run('pkill', ['-f', 'proot']);
    } catch (_) {}
    try {
      await Process.run('killall', ['codedroid_api']);
      await Process.run('killall', ['proot']);
    } catch (_) {}
    await Future.delayed(const Duration(milliseconds: 500));
  }

  static bool get isRunning => _apiProcess != null;
}
