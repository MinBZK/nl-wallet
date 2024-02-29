/// Wrapper for consumable values
class Consumable<T> {
  final T _value;
  bool consumed = false;

  T? get value {
    if (consumed) return null;
    consumed = true;
    return _value;
  }

  Consumable(this._value);

  /// Check the value without consuming it.
  /// Returns null if the value has already been consumed
  T? peek() => consumed ? null : _value;
}
