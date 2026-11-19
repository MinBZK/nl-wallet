import 'package:material_ui/material_ui.dart';

/// Provide the active (i.e. the language used to currently render the app) [Locale].
abstract class ActiveLocaleProvider {
  Stream<Locale> observe();

  Locale get activeLocale;
}
