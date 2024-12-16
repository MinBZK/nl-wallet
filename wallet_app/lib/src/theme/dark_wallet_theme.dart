// ignore_for_file: unused_field

import 'package:flutter/material.dart';

import '../util/extension/color_extension.dart';
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
    surface: Color(0xFF1C1E25),
    inverseSurface: Color(0xFF414966),
    primaryContainer: Color(0xFF2F3444),
    onPrimaryContainer: textColor,
    secondaryContainer: Color(0xFF004785),
    tertiaryContainer: _Colors.pageGutter,
    onPrimary: Color(0xFF002C71),
    onSurface: primaryColorDark,
    onSurfaceVariant: Color(0xFF8292CC),
    outlineVariant: Color(0xFF656D87),
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
    backgroundColor: colorScheme.surface,
    surfaceTintColor: colorScheme.surface,
    iconTheme: const IconThemeData(color: primary, size: 24),
    titleTextStyle: textTheme.displayMedium,
    shadowColor: colorScheme.shadow,
  );

  static final bottomNavigationBarTheme = BaseWalletTheme.baseBottomNavigationBarThemeData.copyWith(
    backgroundColor: colorScheme.surface,
    unselectedItemColor: bottomNavigationUnselectedColor,
  );

  static final elevatedButtonTheme = ElevatedButtonThemeData(
    style: BaseWalletTheme.baseElevatedButtonTheme.style?.copyWith(
      textStyle: WidgetStateTextStyle.resolveWith(
        (states) {
          return BaseWalletTheme.buttonTextStyle.copyWith(
            decoration: states.isHoveredOrFocused ? TextDecoration.underline : null,
          );
        },
      ),
      backgroundColor: WidgetStateProperty.resolveWith(
        (states) {
          if (states.isHoveredOrFocused) return colorScheme.primary.darken();
          return colorScheme.primary;
        },
      ),
      foregroundColor: WidgetStatePropertyAll(colorScheme.onPrimary),
      overlayColor: WidgetStateProperty.resolveWith(
        (states) {
          if (states.isHoveredOrFocused) return colorScheme.secondary.darken();
          return colorScheme.secondary;
        },
      ),
    ),
  );

  static final outlinedButtonTheme = OutlinedButtonThemeData(
    style: BaseWalletTheme.outlinedButtonTheme.style?.copyWith(
      textStyle: WidgetStateTextStyle.resolveWith(
        (states) {
          return BaseWalletTheme.buttonTextStyle.copyWith(
            decoration: states.isHoveredOrFocused ? TextDecoration.underline : null,
          );
        },
      ),
      side: WidgetStatePropertyAll(BorderSide(color: colorScheme.primary, width: 0.5)),
    ),
  );

  static final textButtonTheme = TextButtonThemeData(
    style: BaseWalletTheme.textButtonTheme.style?.copyWith(
      textStyle: WidgetStateTextStyle.resolveWith(
        (states) {
          return BaseWalletTheme.buttonTextStyle.copyWith(
            letterSpacing: 1.15,
            decoration: states.isHoveredOrFocused ? TextDecoration.underline : null,
          );
        },
      ),
      foregroundColor: WidgetStatePropertyAll(colorScheme.primary),
    ),
  );

  static final tabBarTheme = BaseWalletTheme.tabBarTheme.copyWith(
    labelColor: colorScheme.primary,
    unselectedLabelColor: colorScheme.onSurface,
    indicatorColor: colorScheme.primary,
  );

  static final scrollBarTheme = BaseWalletTheme.baseScrollbarTheme.copyWith(
    thumbColor: const WidgetStatePropertyAll(primaryColorDark),
  );

  static final bottomSheetTheme = BaseWalletTheme.baseBottomSheetTheme.copyWith(
    backgroundColor: sheetBackgroundColor,
    surfaceTintColor: Colors.transparent,
  );

  static final iconTheme = BaseWalletTheme.baseIconTheme.copyWith(color: colorScheme.onSurface);

  //endregion Modified (colored) BaseThemes

  static const progressIndicatorTheme = ProgressIndicatorThemeData(linearTrackColor: Color(0xFF292D3A));

  static const focusColor = Colors.white12;
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
