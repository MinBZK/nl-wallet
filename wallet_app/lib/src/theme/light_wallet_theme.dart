import 'package:flutter/material.dart';

import 'base_wallet_theme.dart';

class LightWalletTheme {
  LightWalletTheme._();

  // ColorScheme
  static const colorScheme = ColorScheme.light(
    brightness: Brightness.light,
    primary: primary,
    secondary: Color(0x332065E0),
    error: Color(0xFFAB0065),
    background: Color(0xFFFCFCFC),
    inverseSurface: Color(0xFFEBE4FD),
    //TODO: See if this is appropriate
    secondaryContainer: Color(0xFFF3F4F7),
    onPrimary: Color(0xFFFCFCFC),
    onBackground: primaryColorDark,
    onSurface: Color(0xFF445581),
    outlineVariant: Color(0xFFE8EAEF),
    shadow: Color(0x14000000),
  );

  // Other Colors
  static const primary = Color(0xFF383EDE);
  static const primaryColorDark = Color(0xFF152A62);
  static const sheetBackgroundColor = Color(0xFFFFFFFF);
  static const textColor = primaryColorDark;
  static const bottomNavigationUnselectedColor = Color(0xFF445581);

  // TextTheme
  static final textTheme = BaseWalletTheme.baseTextTheme.apply(
    bodyColor: textColor,
    displayColor: textColor,
  );

  // DialogTheme
  static final dialogTheme = DialogTheme(
    backgroundColor: sheetBackgroundColor,
    titleTextStyle: textTheme.headlineSmall,
    surfaceTintColor: Colors.transparent,
  );

  //region Modified (colored) BaseThemes
  static final dividerTheme = BaseWalletTheme.baseDividerTheme.copyWith(
    color: colorScheme.outlineVariant,
  );

  static final appBarTheme = BaseWalletTheme.baseAppBarTheme.copyWith(
    backgroundColor: colorScheme.background,
    shape: Border(bottom: BorderSide(color: colorScheme.outlineVariant)),
    iconTheme: const IconThemeData(color: primary),
    titleTextStyle: textTheme.titleMedium,
  );

  static final bottomNavigationBarTheme = BaseWalletTheme.baseBottomNavigationBarThemeData.copyWith(
    backgroundColor: colorScheme.background,
    unselectedItemColor: bottomNavigationUnselectedColor,
  );

  static final elevatedButtonTheme = ElevatedButtonThemeData(
    style: BaseWalletTheme.baseElevatedButtonTheme.style?.copyWith(
      foregroundColor: MaterialStatePropertyAll(colorScheme.onPrimary),
      backgroundColor: MaterialStatePropertyAll(colorScheme.primary),
    ),
  );

  static final outlinedButtonTheme = OutlinedButtonThemeData(
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
    surfaceTintColor: Colors.transparent,
  );

  //endregion Modified (colored) BaseThemes

  static final iconTheme = IconThemeData(color: colorScheme.onBackground);

  static const progressIndicatorTheme = ProgressIndicatorThemeData(linearTrackColor: Color(0xFFF2F2FA));
}
