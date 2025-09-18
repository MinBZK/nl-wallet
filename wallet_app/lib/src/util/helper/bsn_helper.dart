class BsnHelper {
  static const kBsnLength = 9;
  static const kAltBsnLength = 8;

  BsnHelper._();

  /// Check if the provided string conforms to the BSN format, this does
  /// not imply the BSN is valid and in use, but does provide us with a
  /// decent amount of certainty that the [bsn] is in fact a BSN and thus
  /// should be treated as such in the UI / by screen-readers.
  static bool isValidBsnFormat(String bsn) {
    String input = bsn.trim();
    if (input.length == kAltBsnLength) input = '0$input';
    if (input.length != kBsnLength) return false;
    try {
      // 11-proef (https://nl.wikipedia.org/wiki/Burgerservicenummer#11-proef)
      final digits = input.split('').map(int.parse).toList();
      final x =
          0 + // for formatter
          (9 * digits[0]) +
          (8 * digits[1]) +
          (7 * digits[2]) +
          (6 * digits[3]) +
          (5 * digits[4]) +
          (4 * digits[5]) +
          (3 * digits[6]) +
          (2 * digits[7]) +
          (-1 * digits[8]);
      return x % 11 == 0;
    } catch (ex) {
      return false;
    }
  }
}
