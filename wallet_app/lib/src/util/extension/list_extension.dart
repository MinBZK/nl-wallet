import 'dart:collection';

extension ListExtension<T extends Object> on List<T?> {
  List<T> get nonNullsList => nonNulls.toList();
}
