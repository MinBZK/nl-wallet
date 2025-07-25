import 'package:flutter/material.dart';

import 'base_wallet_theme.dart';
import 'dark_wallet_theme.dart';
import 'light_wallet_theme.dart';

class WalletTheme {
  const WalletTheme._();

  static const kBorderRadius12 = BorderRadius.all(Radius.circular(12));

  static final ThemeData _baseTheme = ThemeData(
    useMaterial3: true,
    fontFamily: BaseWalletTheme.fontFamily,
    floatingActionButtonTheme: BaseWalletTheme.floatingActionButtonTheme,
    sliderTheme: const SliderThemeData(
      inactiveTrackColor: Color(0xFFABABAB),
      activeTrackColor: Colors.white,
      thumbColor: Colors.white,
      trackHeight: 4,
      thumbShape: RoundSliderThumbShape(enabledThumbRadius: 24 / 2),
      overlayShape: RoundSliderOverlayShape(overlayRadius: 50 / 2),
      overlayColor: Color(0xE61C1E25),
      padding: EdgeInsets.symmetric(horizontal: 2 /* provide tiny spacing for focused state */),
    ),
  );

  static ThemeData light = _baseTheme.copyWith(
    appBarTheme: LightWalletTheme.appBarTheme,
    bottomNavigationBarTheme: LightWalletTheme.bottomNavigationBarTheme,
    bottomSheetTheme: LightWalletTheme.bottomSheetTheme,
    brightness: Brightness.light,
    colorScheme: LightWalletTheme.colorScheme,
    dialogTheme: LightWalletTheme.dialogTheme,
    dividerTheme: LightWalletTheme.dividerTheme,
    elevatedButtonTheme: LightWalletTheme.elevatedButtonTheme,
    focusColor: LightWalletTheme.focusColor,
    iconButtonTheme: LightWalletTheme.iconButtonTheme,
    iconTheme: LightWalletTheme.iconTheme,
    outlinedButtonTheme: LightWalletTheme.outlinedButtonTheme,
    primaryColorDark: LightWalletTheme.primaryColorDark,
    progressIndicatorTheme: LightWalletTheme.progressIndicatorTheme,
    scaffoldBackgroundColor: LightWalletTheme.colorScheme.surface,
    scrollbarTheme: LightWalletTheme.scrollBarTheme,
    tabBarTheme: LightWalletTheme.tabBarTheme,
    textButtonTheme: LightWalletTheme.textButtonTheme,
    textTheme: LightWalletTheme.textTheme,
    cardColor: LightWalletTheme.colorScheme.surface,
  );

  static ThemeData dark = _baseTheme.copyWith(
    appBarTheme: DarkWalletTheme.appBarTheme,
    bottomNavigationBarTheme: DarkWalletTheme.bottomNavigationBarTheme,
    bottomSheetTheme: DarkWalletTheme.bottomSheetTheme,
    brightness: Brightness.dark,
    colorScheme: DarkWalletTheme.colorScheme,
    dialogTheme: DarkWalletTheme.dialogTheme,
    dividerTheme: DarkWalletTheme.dividerTheme,
    elevatedButtonTheme: DarkWalletTheme.elevatedButtonTheme,
    focusColor: DarkWalletTheme.focusColor,
    iconButtonTheme: DarkWalletTheme.iconButtonTheme,
    iconTheme: DarkWalletTheme.iconTheme,
    outlinedButtonTheme: DarkWalletTheme.outlinedButtonTheme,
    primaryColorDark: DarkWalletTheme.primaryColorDark,
    progressIndicatorTheme: DarkWalletTheme.progressIndicatorTheme,
    scaffoldBackgroundColor: DarkWalletTheme.colorScheme.surface,
    scrollbarTheme: DarkWalletTheme.scrollBarTheme,
    tabBarTheme: DarkWalletTheme.tabBarTheme,
    textButtonTheme: DarkWalletTheme.textButtonTheme,
    textTheme: DarkWalletTheme.textTheme,
    cardColor: DarkWalletTheme.colorScheme.surface,
  );
}
