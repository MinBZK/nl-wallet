import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

extension BuildContextExtension on BuildContext {
  bool get isLandscape => MediaQuery.of(this).orientation == Orientation.landscape;

  ThemeData get theme => Theme.of(this);

  TextTheme get textTheme => theme.textTheme;

  ColorScheme get colorScheme => theme.colorScheme;

  AppLocalizations get l10n => AppLocalizations.of(this);
}
