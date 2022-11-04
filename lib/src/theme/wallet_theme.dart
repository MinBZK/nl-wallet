import 'package:flutter/material.dart';

import 'wallet_theme_light_constants.dart';

class WalletTheme {
  const WalletTheme._();

  static ThemeData light = ThemeData(
    colorScheme: WalletThemeConstants.colorScheme,
    fontFamily: WalletThemeConstants.fontFamily,
    indicatorColor: WalletThemeConstants.indicatorColor,
    dividerColor: WalletThemeConstants.dividerColor,
    scaffoldBackgroundColor: WalletThemeConstants.scaffoldBackgroundColor,
    appBarTheme: WalletThemeConstants.appBarTheme,
    bottomNavigationBarTheme: WalletThemeConstants.bottomNavigationBarThemeData,
    elevatedButtonTheme: WalletThemeConstants.elevatedButtonTheme,
    floatingActionButtonTheme: WalletThemeConstants.floatingActionButtonTheme,
    outlinedButtonTheme: WalletThemeConstants.outlinedButtonTheme,
    textButtonTheme: WalletThemeConstants.textButtonTheme,
    tabBarTheme: WalletThemeConstants.tabBarTheme,
    textTheme: WalletThemeConstants.textTheme,
  );
}
