import 'dart:io';
import 'dart:convert';
import 'dart:typed_data';
import 'path_utils.dart';

/// Pure-Dart tar archive extractor.
/// Handles regular files, symlinks (absolute → relative), hardlinks, and directories.
/// Skips Alpine APK metadata and PaxHeaders automatically.
class TarExtractor {
  TarExtractor._();

  static Future<void> extract(String tarPath, String destDir) async {
    final bytes = await File(tarPath).readAsBytes();
    int offset = 0;

    while (offset + 512 <= bytes.length) {
      final header = bytes.sublist(offset, offset + 512);
      // End-of-archive sentinel: two 512-byte zero blocks
      if (header.every((b) => b == 0)) break;

      String name = _parseString(header, 0, 100);
      final prefix = _parseString(header, 345, 155);
      if (prefix.isNotEmpty) name = '$prefix/$name';

      final size = _parseOctal(header, 124, 12);
      final typeflag = String.fromCharCode(header[156]);
      final linkname = _parseString(header, 157, 100);

      offset += 512;

      // Skip PAX and global headers
      if (typeflag == 'x' ||
          typeflag == 'g' ||
          name.startsWith('PaxHeaders/') ||
          name.contains('/PaxHeaders/')) {
        offset += size + _padding(size);
        continue;
      }

      // Skip Alpine APK metadata files
      if (name == '.PKGINFO' || name.startsWith('.SIGN.')) {
        offset += size + _padding(size);
        continue;
      }

      final targetPath = PathUtils.join(destDir, name);

      switch (typeflag) {
        case '5': // Directory
          await Directory(targetPath).create(recursive: true);
          break;

        case '0':
        case '\x00': // Regular file
          await _cleanPath(targetPath);
          await Directory(PathUtils.dirname(targetPath)).create(recursive: true);
          await File(targetPath).writeAsBytes(bytes.sublist(offset, offset + size));
          break;

        case '2': // Symlink
          await _cleanPath(targetPath);
          await Directory(PathUtils.dirname(targetPath)).create(recursive: true);
          final resolvedTarget = _resolveSymlinkTarget(linkname, name);
          await Link(targetPath).create(resolvedTarget);
          break;

        case '1': // Hardlink — copy the source file
          await _cleanPath(targetPath);
          await Directory(PathUtils.dirname(targetPath)).create(recursive: true);
          final sourcePath = PathUtils.join(destDir, linkname);
          final sourceFile = File(sourcePath);
          if (await sourceFile.exists()) {
            await sourceFile.copy(targetPath);
          }
          break;

        default:
          break;
      }

      offset += size + _padding(size);
    }
  }

  // ─── Helpers ────────────────────────────────────────────────────────────────

  static int _padding(int size) => (512 - (size % 512)) % 512;

  static Future<void> _cleanPath(String path) async {
    try {
      if (FileSystemEntity.isLinkSync(path)) {
        await Link(path).delete();
      } else if (File(path).existsSync()) {
        await File(path).delete();
      } else if (Directory(path).existsSync()) {
        await Directory(path).delete(recursive: true);
      }
    } catch (_) {}
  }

  /// Converts absolute symlink targets to relative paths so they work inside PRoot.
  static String _resolveSymlinkTarget(String linkname, String entryName) {
    if (!linkname.startsWith('/')) return linkname;

    final linkDir = PathUtils.dirname(entryName);
    final components = linkDir.split('/');
    int levels = components.length;
    if (linkDir.isEmpty || linkDir == '.') levels = 0;

    final upPrefix = levels == 0 ? '.' : List.generate(levels, (_) => '..').join('/');
    return '$upPrefix/${linkname.substring(1)}';
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
      if (b < 48 || b > 55) continue;
      value = (value << 3) + (b - 48);
    }
    return value;
  }
}
