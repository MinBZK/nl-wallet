extension StringExtension on String {
  /// Capitalizes first letter in string (if present)
  String capitalize() => isNotEmpty ? '${this[0].toUpperCase()}${substring(1).toLowerCase()}' : '';

  /// Removes last character from string (if present)
  String removeLastChar() => length <= 1 ? '' : substring(0, length - 1);

  /// Adds space to end of string (if not empty)
  String addSpaceSuffix() => isNotEmpty ? '$this ' : '';
}
