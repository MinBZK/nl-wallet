import 'package:flutter/material.dart';

import 'base_wallet_theme.dart';

class DarkWalletTheme {
  DarkWalletTheme._();

  // ColorScheme
  static const colorScheme = ColorScheme.dark(
    brightness: Brightness.dark,
    primary: Color(0xFFB1C5FF),
    onPrimary: Color(0xFF002C71),
    secondary: Color(0xFFA5C8FF),
    secondaryContainer: Color(0xFF004785),
    error: Color(0xFFFFB4AB),
    background: Color(0xFF001F25),
    onBackground: primaryColorDark,
    onSurface: Color(0xFFA6EEFF),
    outlineVariant: Color(0xFF44464F),
  );

  // Other Colors
  static const primaryColorDark = Color(0xFFead59d);
  static const sheetBackgroundColor = Color(0xFF03282F);
  static const textColor = primaryColorDark;

  // TextTheme
  static final textTheme = BaseWalletTheme.baseTextTheme.apply(
    bodyColor: textColor,
    displayColor: textColor,
  );

  //region Modified (colored) BaseThemes
  static final dividerTheme = BaseWalletTheme.baseDividerTheme.copyWith(
    color: colorScheme.outlineVariant,
  );

  static final appBarTheme = BaseWalletTheme.baseAppBarTheme.copyWith(
    backgroundColor: colorScheme.background,
    shape: Border(bottom: BorderSide(color: colorScheme.outlineVariant)),
    iconTheme: IconThemeData(color: colorScheme.onBackground),
    titleTextStyle: textTheme.titleMedium,
  );

  static final bottomNavigationBarTheme = BaseWalletTheme.baseBottomNavigationBarThemeData.copyWith(
    backgroundColor: colorScheme.background,
  );

  static final elevatedButtonTheme = ElevatedButtonThemeData(
    style: BaseWalletTheme.baseElevatedButtonTheme.style?.copyWith(
      foregroundColor: MaterialStatePropertyAll(colorScheme.onPrimary),
      backgroundColor: MaterialStatePropertyAll(colorScheme.primary),
    ),
  );

  static final outlinedButtonTHeme = OutlinedButtonThemeData(
    style: BaseWalletTheme.outlinedButtonTheme.style?.copyWith(
      side: MaterialStatePropertyAll(BorderSide(color: colorScheme.primary, width: 0.5)),
    ),
  );

  static final textButtonTheme = TextButtonThemeData(
    style: BaseWalletTheme.textButtonTheme.style?.copyWith(
      textStyle: MaterialStatePropertyAll(BaseWalletTheme.buttonTextStyle.copyWith(letterSpacing: 1.15)),
      foregroundColor: MaterialStatePropertyAll(colorScheme.primary),
    ),
  );

  static final tabBarTheme = BaseWalletTheme.tabBarTheme.copyWith(
    labelColor: colorScheme.primary,
    unselectedLabelColor: colorScheme.onBackground,
    indicatorColor: colorScheme.primary,
  );

  static final scrollBarTheme = BaseWalletTheme.baseScrollbarTheme.copyWith(
    thumbColor: const MaterialStatePropertyAll(primaryColorDark),
  );

  static final bottomSheetTheme = BaseWalletTheme.baseBottomSheetTheme.copyWith(
    backgroundColor: sheetBackgroundColor,
  );

//endregion Modified (colored) BaseThemes
}
