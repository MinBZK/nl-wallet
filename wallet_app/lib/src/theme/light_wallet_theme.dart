// ignore_for_file: unused_field

import 'package:flutter/material.dart';

import 'colored_wallet_theme.dart';

class LightWalletTheme {
  LightWalletTheme._();

  // ColorScheme
  static const colorScheme = ColorScheme.light(
    brightness: Brightness.light,
    primary: _Colors.primary,
    secondary: Color(0x332065E0),
    onSecondary: Color(0XFF383EDE),
    error: _Colors.textError,
    surface: _Colors.pageBackground,
    inverseSurface: Color(0xFFEBE4FD),
    primaryContainer: _Colors.pageContainers,
    onPrimaryContainer: _Colors.textPrimary,
    secondaryContainer: Color(0xFFF3F4F7),
    tertiaryContainer: _Colors.pageGutter,
    onPrimary: Color(0xFFFCFCFC),
    onSurface: _Colors.textPrimary,
    onSurfaceVariant: _Colors.textSecondary,
    outlineVariant: _Colors.pageSpacer,
    shadow: Color(0x14000000),
  );

  static final theme = ColoredWalletTheme(
    WalletThemeColors(
      colorScheme: colorScheme,
      textColor: _Colors.textPrimary,
      focusColor: _Colors.actionPrimaryBgHover,
      bottomNavigationUnselectedColor: _Colors.inactive,
      modalBackground: _Colors.modalBg,
      modalBarrier: _Colors.modalBarrierColor,
      actionPrimaryHover: _Colors.actionPrimaryHover,
      actionPrimaryBackgroundHover: _Colors.actionPrimaryBgHover,
      actionSecondary: _Colors.actionSecondary,
      elevatedButtonForeground: colorScheme.onPrimary,
    ),
  );

  static final textTheme = theme.textTheme;
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
  static const Color actionPrimaryBg = Colors.transparent;
  static const Color actionPrimaryHover = Color(0xFF0C1195);
  static const Color actionFocused = Color(0xFF3237C4);
  static const Color actionPrimaryBgHover = Color(0x1A7F8CB0);

  // Modals (dialogs & bottom sheets)
  static const Color modalBg = Color(0xFFFCFCFC);
  static const Color modalBarrierColor = Color(0x33152A62);

  // Pages
  static const Color pageOverlay = Color(0xFFFFFFFF);
  static const Color pagePlaceholder = Color(0xFFA1AAC0);
  static const Color pageContainers = Color(0xFFF1F5FF);
  static const Color pageGutter = Color(0xFFF2F2FA);
  static const Color pageBackground = Color(0xFFFCFCFC);
  static const Color pageSpacer = Color(0xFFE8EAEF);
}
