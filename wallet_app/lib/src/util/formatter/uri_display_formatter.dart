class UriDisplayFormatter {
  /// Formats a URL for display purposes, stripping the scheme.
  ///
  /// Returns the host and path of [uri] (e.g. `example.com/path`), or the original
  /// [uri] unmodified if it can't be parsed as a URI.
  static String format(String uri) {
    final parsedUri = Uri.tryParse(uri);
    return parsedUri != null ? (parsedUri.host + parsedUri.path) : uri;
  }
}
