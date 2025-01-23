class SemanticsHelper {
  SemanticsHelper._();

  /// If the input solely consists of positive digits, this method
  /// will return a space separated list of those digits. Useful to
  /// make a screen-reader read out the digits separately.
  /// Otherwise it returns the original input.
  /// OPTIMIZE: Discovered [SpellOutStringAttribute] which is likely a nicer solution.
  static String splitNumberString(String input) {
    if (input.trim().length != input.length) return input;
    final isPositiveInt = int.tryParse(input)?.isNegative == false;
    if (isPositiveInt) return input.split('').join(' ');
    return input;
  }
}
