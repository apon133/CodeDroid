import 'dart:io';
import 'package:flutter/foundation.dart';

/// Handles HTTP file downloads with:
///  - configurable timeout
///  - exponential-backoff retries
///  - optional progress callbacks
class DownloadService {
  DownloadService._();

  static const int _maxRetries = 3;
  static const Duration _connectTimeout = Duration(seconds: 30);
  static const Duration _receiveTimeout = Duration(minutes: 10);

  /// Downloads [url] to [savePath].
  ///
  /// [onProgress] receives a 0.0–1.0 fraction if the server sends Content-Length,
  /// otherwise it is called with -1 on each chunk to signal indeterminate progress.
  ///
  /// Throws on permanent failure after [_maxRetries] attempts.
  static Future<void> download(
    String url,
    String savePath, {
    void Function(double progress, int downloaded, int? total)? onProgress,
  }) async {
    for (int attempt = 1; attempt <= _maxRetries; attempt++) {
      try {
        await _attemptDownload(url, savePath, onProgress: onProgress);
        return; // success
      } catch (e) {
        debugPrint('DownloadService: attempt $attempt/$_maxRetries failed for $url: $e');
        if (attempt == _maxRetries) {
          rethrow;
        }
        // Exponential back-off: 2s, 4s, 8s …
        await Future.delayed(Duration(seconds: 2 * attempt));
      }
    }
  }

  static Future<void> _attemptDownload(
    String url,
    String savePath, {
    void Function(double, int, int?)? onProgress,
  }) async {
    final client = HttpClient()
      ..connectionTimeout = _connectTimeout
      ..idleTimeout = _receiveTimeout;

    try {
      final request = await client.getUrl(Uri.parse(url));
      final response = await request.close();

      if (response.statusCode != 200) {
        throw HttpException(
          'Server returned ${response.statusCode} for $url',
        );
      }

      final int? total = response.contentLength >= 0 ? response.contentLength : null;
      int downloaded = 0;

      final file = File(savePath);
      final sink = file.openWrite();

      await for (final chunk in response) {
        sink.add(chunk);
        downloaded += chunk.length;
        if (onProgress != null) {
          final progress = total != null ? downloaded / total : -1.0;
          onProgress(progress, downloaded, total);
        }
      }

      await sink.flush();
      await sink.close();
    } finally {
      client.close();
    }
  }

  /// Decompresses a gzip-compressed APK into a plain .tar file using system gzip.
  static Future<void> decompressGzip(String gzipPath, String tarPath) async {
    final result = await Process.run('sh', [
      '-c',
      "gzip -d -c -f '${_esc(gzipPath)}' > '${_esc(tarPath)}'"
    ]);
    if (result.exitCode != 0) {
      throw Exception('gzip decompress failed (exit ${result.exitCode}): ${result.stderr}');
    }
  }

  static String _esc(String s) => s.replaceAll("'", "'\\''");
}
