// ignore_for_file: unused_field

import 'package:flutter/material.dart';

import 'base_wallet_theme.dart';

class LightWalletTheme {
  LightWalletTheme._();

  // ColorScheme
  static const colorScheme = ColorScheme.light(
    brightness: Brightness.light,
    primary: _Colors.primary,
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
  static const bottomNavigationUnselectedColor = Color(0xFF445581);
  static const focusColor = _Colors.actionPrimaryBgHover;
  static const primaryColorDark = Color(0xFF152A62);
  static const textColor = primaryColorDark;

  // TextTheme
  static final textTheme = BaseWalletTheme.baseTextTheme.apply(
    bodyColor: textColor,
    displayColor: textColor,
  );

  // DialogTheme
  static final dialogTheme = DialogThemeData(
    titleTextStyle: textTheme.headlineMedium,
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
    titleTextStyle: textTheme.headlineMedium,
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
      foregroundColor: WidgetStatePropertyAll(colorScheme.onPrimary),
      iconColor: WidgetStatePropertyAll(colorScheme.onPrimary),
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

  static const progressIndicatorTheme = ProgressIndicatorThemeData(linearTrackColor: Color(0xFFF2F2FA));
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
  static const Color primary = Color(0xFF383EDE);

  // Icons
  static const Color inactive = Color(0xFF445581);
  static const Color iconsAction = Color(0xFF383EDE);
  static const Color iconsWhite = Color(0xFFFFFFFF);

  // Text
  static const Color textAction = Color(0xFF383EDE);
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
  static const Color actionPrimaryBg = Color(0xFFFCFCFC);
  static const Color actionPrimaryHover = Color(0xFF0C1195);
  static const Color actionFocused = Color(0xFF3237C4);
  static const Color actionPrimaryBgHover = Color(0x1A7F8CB0);

  // Pages
  static const Color pageOverlay = Color(0xFFFFFFFF);
  static const Color pagePlaceholder = Color(0xFFA1AAC0);
  static const Color pageContainers = Color(0xFFF1F5FF);
  static const Color pageGutter = Color(0xFFF2F2FA);
  static const Color pageBackground = Color(0xFFFCFCFC);
  static const Color pageSpacer = Color(0xFFE8EAEF);
}
