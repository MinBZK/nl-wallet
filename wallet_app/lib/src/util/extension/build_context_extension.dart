import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

extension BuildContextExtension on BuildContext {
  MediaQueryData get mediaQuery => MediaQuery.of(this);

  TextScaler get textScaler => mediaQuery.textScaler;

  /// Checks whether the device is currently rendering the app in landscape mode
  ///
  /// Test note: When running tests with the deviceBuilder this returns true! Because the canvas to place
  /// all the devices on is used to check the orientation (and it's wide).
  bool get isLandscape => mediaQuery.orientation == Orientation.landscape;

  bool get isScreenReaderEnabled => mediaQuery.accessibleNavigation;

  ThemeData get theme => Theme.of(this);

  TextTheme get textTheme => theme.textTheme;

  ColorScheme get colorScheme => theme.colorScheme;

  AppLocalizations get l10n => AppLocalizations.of(this);

  String get locale => l10n.localeName;
}
