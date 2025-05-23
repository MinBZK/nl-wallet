import 'dart:ui' show Locale;

export 'dart:ui' show Locale;

/// A map type that associates localized text strings with their corresponding locales.
///
/// This type is used to store translations of a text in multiple languages.
/// Each key is a [Locale] representing a language/region, and each value is
/// the translated string for that locale.
///
/// Extension methods provide functionality to:
/// - Retrieve the most appropriate translation based on the active locale
/// - Fall back to alternative translations when the exact locale match isn't available
typedef LocalizedText = Map<Locale, String /*value*/ >;
