import 'package:collection/collection.dart';
import 'package:country_codes/country_codes.dart';
import 'package:fimber/fimber.dart';

class CountryCodeFormatter {
  /// Takes an [ISO-3166-1 alpha-2] country code and returns the name of the associated country in the active locale.
  ///
  /// Returns null if the country can't be resolved or
  /// returns an unlocalized country name if no localized label is found.
  static String? format(String? countryCode) {
    if (countryCode == null) return null;
    try {
      final details = CountryCodes.countryCodes().firstWhereOrNull(
        (details) => details.alpha2Code?.toUpperCase() == countryCode.toUpperCase(),
      );
      return details?.localizedName ?? details!.name!;
    } catch (exception) {
      Fimber.e('Failed to resolve country label from code: $countryCode', ex: exception);
      return null;
    }
  }
}
