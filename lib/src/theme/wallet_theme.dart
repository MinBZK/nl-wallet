import 'package:flutter/material.dart';

import 'base_wallet_theme.dart';
import 'dark_wallet_theme.dart';
import 'light_wallet_theme.dart';

class WalletTheme {
  const WalletTheme._();

  static final ThemeData _baseTheme = ThemeData(
    useMaterial3: true,
    fontFamily: BaseWalletTheme.fontFamily,
    floatingActionButtonTheme: BaseWalletTheme.floatingActionButtonTheme,
  );

  static ThemeData light = _baseTheme.copyWith(
    colorScheme: LightWalletTheme.colorScheme,
    primaryColorDark: LightWalletTheme.primaryColorDark,
    dividerTheme: LightWalletTheme.dividerTheme,
    appBarTheme: LightWalletTheme.appBarTheme,
    bottomNavigationBarTheme: LightWalletTheme.bottomNavigationBarTheme,
    elevatedButtonTheme: LightWalletTheme.elevatedButtonTheme,
    outlinedButtonTheme: LightWalletTheme.outlinedButtonTHeme,
    textButtonTheme: LightWalletTheme.textButtonTheme,
    tabBarTheme: LightWalletTheme.tabBarTheme,
    textTheme: LightWalletTheme.textTheme,
    scrollbarTheme: LightWalletTheme.scrollBarTheme,
    bottomSheetTheme: LightWalletTheme.bottomSheetTheme,
    scaffoldBackgroundColor: LightWalletTheme.colorScheme.background,
  );

  static ThemeData dark = _baseTheme.copyWith(
    colorScheme: DarkWalletTheme.colorScheme,
    primaryColorDark: DarkWalletTheme.primaryColorDark,
    dividerTheme: DarkWalletTheme.dividerTheme,
    appBarTheme: DarkWalletTheme.appBarTheme,
    bottomNavigationBarTheme: DarkWalletTheme.bottomNavigationBarTheme,
    elevatedButtonTheme: DarkWalletTheme.elevatedButtonTheme,
    outlinedButtonTheme: DarkWalletTheme.outlinedButtonTHeme,
    textButtonTheme: DarkWalletTheme.textButtonTheme,
    tabBarTheme: DarkWalletTheme.tabBarTheme,
    textTheme: DarkWalletTheme.textTheme,
    scrollbarTheme: DarkWalletTheme.scrollBarTheme,
    bottomSheetTheme: DarkWalletTheme.bottomSheetTheme,
    scaffoldBackgroundColor: DarkWalletTheme.colorScheme.background,
  );
}
