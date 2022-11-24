extension StringExtension on String {
  /// Capitalizes first letter in string (if present)
  String capitalize() => isNotEmpty ? '${this[0].toUpperCase()}${substring(1).toLowerCase()}' : '';

  String removeLastChar() => length <= 1 ? '' : substring(0, length - 1);
}
