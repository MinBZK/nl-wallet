import 'dart:collection';

extension NullableListExtension<T extends Object> on List<T?> {
  List<T> get nonNullsList => nonNulls.toList();
}

extension ListExtension<T extends Object> on List<T> {
  /// Replaces an element in the list with a new element based on a stable ID.
  ///
  /// @param replacement The new element to insert into the list.
  /// @param getId A function that takes an element of type `T` and returns its stable ID.
  /// @return A new list with the element replaced, or the original list if no matching element was found.
  List<T> replace(T replacement, int Function(T) getId) {
    // Find the correct index based on a stable id
    final index = indexWhere((it) => getId(it) == getId(replacement));

    // If the element is not found, return the original list.
    if (index < 0) return this;

    // Create a new list with the updated element to ensure immutability.
    return List<T>.from(this)..[index] = replacement;
  }

  /// Converts a list into a map where the keys are the elements of the list
  /// and the values are derived from these elements using the provided mapper function.
  ///
  /// @param mapper A function that takes an element of type `T` from the list
  ///               and returns a value of type `V` to be used as the value in the map.
  /// @return A new map where each element from the original list is a key,
  ///         and the corresponding value is the result of applying the `mapper` function to that element.
  Map<T, V> toMap<V extends Object>(V Function(T) mapper) => {for (final item in this) item: mapper(item)};
}
