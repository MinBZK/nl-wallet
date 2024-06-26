// ignore_for_file: unused_field

import 'package:flutter/material.dart';

import 'base_wallet_theme.dart';

class DarkWalletTheme {
  DarkWalletTheme._();

  // ColorScheme
  static const colorScheme = ColorScheme.dark(
    brightness: Brightness.dark,
    primary: primary,
    secondary: Color(0xFFA5C8FF),
    onSecondary: Color(0XFF383EDE),
    error: Color(0xFFFF8989),
    background: Color(0xFF1C1E25),
    inverseSurface: Color(0xFF414966),
    primaryContainer: Color(0xFF2F3444),
    onPrimaryContainer: textColor,
    secondaryContainer: Color(0xFF004785),
    onPrimary: Color(0xFF002C71),
    onBackground: primaryColorDark,
    onSurface: Color(0xFF8292CC),
    outlineVariant: Color(0xFF33343B),
    shadow: Color(0x14FFFFFF),
  );

  // Other Colors
  static const primary = Color(0xFFA2B7FF);
  static const primaryColorDark = Color(0xFFFFFFFF);
  static const sheetBackgroundColor = Color(0xFF03282F);
  static const textColor = primaryColorDark;
  static const bottomNavigationUnselectedColor = Color(0xFFAAACB3);

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
    shape: RoundedRectangleBorder(
      borderRadius: BorderRadius.circular(16),
    ),
  );

  //region Modified (colored) BaseThemes
  static final dividerTheme = BaseWalletTheme.baseDividerTheme.copyWith(
    color: colorScheme.outlineVariant,
  );

  static final appBarTheme = BaseWalletTheme.baseAppBarTheme.copyWith(
    backgroundColor: colorScheme.background,
    surfaceTintColor: colorScheme.background,
    iconTheme: const IconThemeData(color: primary, size: 24),
    titleTextStyle: textTheme.displayMedium,
    shadowColor: colorScheme.shadow,
  );

  static final bottomNavigationBarTheme = BaseWalletTheme.baseBottomNavigationBarThemeData.copyWith(
    backgroundColor: colorScheme.background,
    unselectedItemColor: bottomNavigationUnselectedColor,
  );

  static final elevatedButtonTheme = ElevatedButtonThemeData(
    style: BaseWalletTheme.baseElevatedButtonTheme.style?.copyWith(
      foregroundColor: MaterialStatePropertyAll(colorScheme.onPrimary),
      backgroundColor: MaterialStatePropertyAll(colorScheme.primary),
      overlayColor: MaterialStatePropertyAll(colorScheme.secondary),
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

  static final iconTheme = IconThemeData(color: colorScheme.onBackground, size: 16);

  static const progressIndicatorTheme = ProgressIndicatorThemeData(linearTrackColor: Color(0xFF292D3A));
}

// ignore: unused_element
class _Colors {
  // Icons
  static const Color inactive = Color(0xFFFFFFFF);
  static const Color iconsAction = Color(0xFFA2B7FF);
  static const Color iconsWhite = Color(0xFF152A62);

  // Text
  static const Color textIcon = Color(0xFFA2B7FF);
  static const Color textWhite = Color(0xFF0D193B);
  static const Color textSecondary = Color(0xFFD0D4E0);
  static const Color textPrimary = Color(0xFFFFFFFF);
  static const Color textError = Color(0xFFFF8989);
  static const Color textAlert = Color(0xFFF4AEFF);

  // Buttons
  static const Color actionSecondary = Color(0xFFA1AAC0);
  static const Color actionActive = Color(0xFF8293CC);
  static const Color actionDestructive = Color(0xFFFF8989);
  static const Color actionPrimary = Color(0xFFA2B7FF);

  // Pages
  static const Color pageOverlay = Color(0xFF1C1E25);
  static const Color pagePlaceholder = Color(0xFF616E99);
  static const Color pageContainers = Color(0xFF2F3444);
  static const Color pageGutter = Color(0xFF292D3A);
  static const Color pageBackground = Color(0xFF1C1E25);
  static const Color pageSpacer = Color(0xFF33343B);
}
