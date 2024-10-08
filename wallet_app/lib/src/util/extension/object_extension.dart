extension GenericExtension<T> on T {
  T? takeIf(bool Function(T) predicate) {
    if (predicate(this)) {
      return this;
    } else {
      return null;
    }
  }

  R let<R>(R Function(T that) op) => op(this);
}
