import 'dart:io';

/// Utility functions for path resolution and canonicalization.
class PathUtils {
  PathUtils._();

  /// Resolves symlinks and normalizes a path. Falls back to input on error.
  static String canonicalize(String path) {
    try {
      final file = File(path);
      if (file.existsSync()) return file.resolveSymbolicLinksSync();
      final dir = Directory(path);
      if (dir.existsSync()) return dir.resolveSymbolicLinksSync();
      // Path doesn't exist yet — canonicalize parent, append basename
      final parent = Directory(path).parent;
      if (parent.existsSync()) {
        return '${parent.resolveSymbolicLinksSync()}/${path.split('/').last}';
      }
    } catch (_) {}
    return path;
  }

  static String join(String a, String b) {
    if (a.isEmpty) return b;
    if (b.isEmpty) return a;
    return a.endsWith('/') ? '$a$b' : '$a/$b';
  }

  static String dirname(String p) {
    final idx = p.lastIndexOf('/');
    return idx == -1 ? '.' : p.substring(0, idx);
  }

  /// Deletes a path whether it is a file or symlink. Returns true on success.
  static bool deleteFileOrLink(String path) {
    try {
      final link = Link(path);
      if (link.existsSync()) {
        link.deleteSync();
        return true;
      }
    } catch (_) {}
    try {
      final file = File(path);
      if (file.existsSync()) {
        file.deleteSync();
        return true;
      }
    } catch (_) {}
    return false;
  }
}
