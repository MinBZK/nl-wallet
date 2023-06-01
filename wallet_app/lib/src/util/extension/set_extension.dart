extension SetExtension<T> on Set<T> {
  /// Adds [value] to set if not present, otherwise removes it
  void toggle(T value) {
    if (contains(value)) {
      remove(value);
    } else {
      add(value);
    }
  }
}
