// ignore_for_file: unused_field

import 'package:flutter/material.dart';

import 'base_wallet_theme.dart';

class DarkWalletTheme {
  DarkWalletTheme._();

  // ColorScheme
  static const colorScheme = ColorScheme.dark(
    brightness: Brightness.dark,
    primary: _Colors.primary,
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
  static const bottomNavigationUnselectedColor = Color(0xFFAAACB3);
  static const focusColor = Colors.white12;
  static const primaryColorDark = Color(0xFFFFFFFF);
  static const textColor = primaryColorDark;

  // TextTheme
  static final textTheme = BaseWalletTheme.baseTextTheme.apply(
    bodyColor: textColor,
    displayColor: textColor,
  );

  // DialogTheme
  static final dialogTheme = DialogTheme(
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
    titleTextStyle: textTheme.displayMedium,
    shadowColor: colorScheme.shadow,
  );

  static final bottomNavigationBarTheme = BaseWalletTheme.baseBottomNavigationBarThemeData.copyWith(
    backgroundColor: colorScheme.surface,
    unselectedItemColor: bottomNavigationUnselectedColor,
  );

  static final elevatedButtonTheme = ElevatedButtonThemeData(
    style: BaseWalletTheme.baseElevatedButtonTheme.style?.copyWith(
      backgroundColor: WidgetStateColor.resolveWith(
        (states) => states.isPressedOrFocused ? _Colors.actionPrimaryHover : colorScheme.primary,
      ),
      foregroundColor: const WidgetStatePropertyAll(_Colors.textWhite),
      iconColor: const WidgetStatePropertyAll(_Colors.textWhite),
      overlayColor: WidgetStateProperty.resolveWith(
        (states) => states.isPressedOrFocused ? _Colors.actionPrimaryHover : null,
      ),
      side: WidgetStateBorderSide.resolveWith(
        (states) => states.isFocused ? _Sides.buttonBorderSideFocused : null,
      ),
    ),
  );

  static final outlinedButtonTheme = OutlinedButtonThemeData(
    style: BaseWalletTheme.baseOutlinedButtonTheme.style?.copyWith(
      backgroundColor: WidgetStateColor.resolveWith(
        (states) => states.isPressedOrFocused ? _Colors.actionPrimaryBgHover : _Colors.actionPrimaryBg,
      ),
      foregroundColor: WidgetStateColor.resolveWith(
        (states) => states.isPressedOrFocused ? _Colors.actionPrimaryHover : _Colors.textAction,
      ),
      iconColor: WidgetStateColor.resolveWith(
        (states) => states.isPressedOrFocused ? _Colors.actionPrimaryHover : _Colors.textAction,
      ),
      overlayColor: WidgetStateProperty.resolveWith(
        (states) => states.isPressedOrFocused ? _Colors.actionPrimaryBgHover : null,
      ),
      side: WidgetStateBorderSide.resolveWith(
        (states) {
          if (states.isPressed) return _Sides.outlineButtonBorderSidePressed;
          if (states.isFocused) return _Sides.buttonBorderSideFocused;
          return _Sides.outlineButtonBorderSideDefault;
        },
      ),
    ),
  );

  static final textButtonTheme = TextButtonThemeData(
    style: BaseWalletTheme.baseTextButtonTheme.style?.copyWith(
      backgroundColor: WidgetStateColor.resolveWith(
        (states) => states.isPressedOrFocused ? _Colors.actionPrimaryBgHover : _Colors.actionPrimaryBg,
      ),
      foregroundColor: WidgetStateColor.resolveWith(
        (states) => states.isPressedOrFocused ? _Colors.actionPrimaryHover : _Colors.textAction,
      ),
      iconColor: WidgetStateColor.resolveWith(
        (states) => states.isPressedOrFocused ? _Colors.actionPrimaryHover : _Colors.textAction,
      ),
      overlayColor: WidgetStateProperty.resolveWith(
        (states) => states.isPressedOrFocused ? _Colors.actionPrimaryBgHover : null,
      ),
      side: WidgetStateBorderSide.resolveWith(
        (states) => states.isFocused ? _Sides.buttonBorderSideFocused : null,
      ),
    ),
  );

  static final iconButtonTheme = IconButtonThemeData(
    style: BaseWalletTheme.baseIconButtonTheme.style?.copyWith(
      foregroundColor: WidgetStateColor.resolveWith(
        (states) => states.isPressedOrFocused ? _Colors.actionPrimaryHover : _Colors.iconsAction,
      ),
      iconColor: WidgetStateColor.resolveWith(
        (states) => states.isPressedOrFocused ? _Colors.actionPrimaryHover : _Colors.iconsAction,
      ),
      side: WidgetStateProperty.resolveWith(
        (states) => states.isFocused ? _Sides.buttonBorderSideFocused : null,
      ),
    ),
  );

  static final tabBarTheme = BaseWalletTheme.baseTabBarTheme.copyWith(
    labelColor: colorScheme.primary,
    unselectedLabelColor: colorScheme.onSurface,
    indicatorColor: colorScheme.primary,
  );

  static final scrollBarTheme = BaseWalletTheme.baseScrollbarTheme.copyWith(
    thumbColor: const WidgetStatePropertyAll(primaryColorDark),
  );

  static final bottomSheetTheme = BaseWalletTheme.baseBottomSheetTheme.copyWith(
    surfaceTintColor: Colors.transparent,
  );

  static final iconTheme = BaseWalletTheme.baseIconTheme.copyWith(color: colorScheme.onSurface);

  //endregion Modified (colored) BaseThemes

  static const progressIndicatorTheme = ProgressIndicatorThemeData(linearTrackColor: Color(0xFF292D3A));
}

class _Sides {
  static final BorderSide buttonBorderSideFocused = BaseWalletTheme.buttonBorderSideFocused.copyWith(
    color: _Colors.actionPrimaryHover,
  );

  static final BorderSide outlineButtonBorderSideDefault = BaseWalletTheme.outlineButtonBorderSideDefault.copyWith(
    color: _Colors.primary,
  );
  static final BorderSide outlineButtonBorderSidePressed = BaseWalletTheme.outlineButtonBorderSideDefault.copyWith(
    color: _Colors.actionPrimaryHover,
  );
}

// ignore: unused_element
class _Colors {
  // Color scheme
  static const Color primary = Color(0xFFA2B7FF);

  // Icons
  static const Color inactive = Color(0xFFFFFFFF);
  static const Color iconsAction = Color(0xFFA2B7FF);
  static const Color iconsWhite = Color(0xFF152A62);

  // Text
  static const Color textAction = Color(0xFFA2B7FF);
  static const Color textWhite = Color(0xFF0D193B);
  static const Color textSecondary = Color(0xFFD0D4E0);
  static const Color textPrimary = Color(0xFFFFFFFF);
  static const Color textError = Color(0xFFFF8989);
  static const Color textAlert = Color(0xFFF4AEFF);

  // Buttons
  static const Color actionSecondary = Color(0xFFA1AAC0);
  static const Color actionActive = Color(0xFF8293CC);
  static const Color actionDestructive = Color(0xFFFF8989);
  static const Color actionDestructiveHover = Color(0xFFFFB7B7);
  static const Color actionPrimary = Color(0xFFA2B7FF);
  static const Color actionPrimaryBg = Color(0xFF1C1E25);
  static const Color actionPrimaryHover = Color(0xFFC8D5FF);
  static const Color actionPrimaryBgHover = Color(0x1A8592B3);

  // Pages
  static const Color pageOverlay = Color(0xFF1C1E25);
  static const Color pagePlaceholder = Color(0xFF616E99);
  static const Color pageContainers = Color(0xFF2F3444);
  static const Color pageGutter = Color(0xFF292D3A);
  static const Color pageBackground = Color(0xFF1C1E25);
  static const Color pageSpacer = Color(0xFF33343B);
}
