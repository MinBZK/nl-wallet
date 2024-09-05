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
    appBarTheme: LightWalletTheme.appBarTheme,
    bottomNavigationBarTheme: LightWalletTheme.bottomNavigationBarTheme,
    bottomSheetTheme: LightWalletTheme.bottomSheetTheme,
    brightness: Brightness.light,
    colorScheme: LightWalletTheme.colorScheme,
    dividerTheme: LightWalletTheme.dividerTheme,
    elevatedButtonTheme: LightWalletTheme.elevatedButtonTheme,
    iconTheme: LightWalletTheme.iconTheme,
    outlinedButtonTheme: LightWalletTheme.outlinedButtonTheme,
    primaryColorDark: LightWalletTheme.primaryColorDark,
    progressIndicatorTheme: LightWalletTheme.progressIndicatorTheme,
    scaffoldBackgroundColor: LightWalletTheme.colorScheme.surface,
    scrollbarTheme: LightWalletTheme.scrollBarTheme,
    tabBarTheme: LightWalletTheme.tabBarTheme,
    textButtonTheme: LightWalletTheme.textButtonTheme,
    textTheme: LightWalletTheme.textTheme,
    dialogTheme: LightWalletTheme.dialogTheme,
    focusColor: LightWalletTheme.focusColor,
  );

  static ThemeData dark = _baseTheme.copyWith(
    appBarTheme: DarkWalletTheme.appBarTheme,
    bottomNavigationBarTheme: DarkWalletTheme.bottomNavigationBarTheme,
    bottomSheetTheme: DarkWalletTheme.bottomSheetTheme,
    brightness: Brightness.dark,
    colorScheme: DarkWalletTheme.colorScheme,
    dividerTheme: DarkWalletTheme.dividerTheme,
    elevatedButtonTheme: DarkWalletTheme.elevatedButtonTheme,
    iconTheme: DarkWalletTheme.iconTheme,
    outlinedButtonTheme: DarkWalletTheme.outlinedButtonTheme,
    primaryColorDark: DarkWalletTheme.primaryColorDark,
    progressIndicatorTheme: DarkWalletTheme.progressIndicatorTheme,
    scaffoldBackgroundColor: DarkWalletTheme.colorScheme.surface,
    scrollbarTheme: DarkWalletTheme.scrollBarTheme,
    tabBarTheme: DarkWalletTheme.tabBarTheme,
    textButtonTheme: DarkWalletTheme.textButtonTheme,
    textTheme: DarkWalletTheme.textTheme,
    dialogTheme: DarkWalletTheme.dialogTheme,
    focusColor: DarkWalletTheme.focusColor,
  );
}
