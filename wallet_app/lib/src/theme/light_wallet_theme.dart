// ignore_for_file: unused_field

import 'package:flutter/material.dart';

import 'base_wallet_theme.dart';

class LightWalletTheme {
  LightWalletTheme._();

  // ColorScheme
  static const colorScheme = ColorScheme.light(
    brightness: Brightness.light,
    primary: primary,
    secondary: Color(0x332065E0),
    onSecondary: Color(0XFF383EDE),
    error: Color(0xFFAB0065),
    surface: Color(0xFFFCFCFC),
    inverseSurface: Color(0xFFEBE4FD),
    primaryContainer: Color(0xFFF1F5FF),
    onPrimaryContainer: textColor,
    secondaryContainer: Color(0xFFF3F4F7),
    tertiaryContainer: _Colors.pageGutter,
    onPrimary: Color(0xFFFCFCFC),
    onSurface: primaryColorDark,
    onSurfaceVariant: Color(0xFF445581),
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
      foregroundColor: WidgetStatePropertyAll(colorScheme.onPrimary),
      backgroundColor: WidgetStateProperty.resolveWith(
        (states) {
          if (states.isHoveredOrFocused) return _Colors.actionFocused;
          return colorScheme.primary;
        },
      ),
      overlayColor: WidgetStateProperty.resolveWith(
        (states) {
          if (states.isHoveredOrFocused) return _Colors.actionFocused;
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

  static const progressIndicatorTheme = ProgressIndicatorThemeData(linearTrackColor: Color(0xFFF2F2FA));

  static const focusColor = Colors.black12;
}

// ignore: unused_element
class _Colors {
  // Icons
  static const Color inactive = Color(0xFF445581);
  static const Color iconsAction = Color(0xFF383EDE);
  static const Color iconsWhite = Color(0xFFFFFFFF);

  // Text
  static const Color textIcon = Color(0xFF383EDE);
  static const Color textWhite = Color(0xFFFFFFFF);
  static const Color textSecondary = Color(0xFF445581);
  static const Color textPrimary = Color(0xFF152A62);
  static const Color textError = Color(0xFFAB0065);
  static const Color textAlert = Color(0xFF9300AB);

  // Buttons
  static const Color actionSecondary = Color(0xFF445581);
  static const Color actionActive = Color(0xFF152A62);
  static const Color actionDestructive = Color(0xFFAB0065);
  static const Color actionPrimary = Color(0xFF383EDE);
  static const Color actionFocused = Color(0xFF3237C4);

  // Pages
  static const Color pageOverlay = Color(0xFFFFFFFF);
  static const Color pagePlaceholder = Color(0xFFA1AAC0);
  static const Color pageContainers = Color(0xFFF1F5FF);
  static const Color pageGutter = Color(0xFFF2F2FA);
  static const Color pageBackground = Color(0xFFFCFCFC);
  static const Color pageSpacer = Color(0xFFE8EAEF);
}
