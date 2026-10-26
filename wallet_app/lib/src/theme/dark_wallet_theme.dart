// ignore_for_file: unused_field

import 'package:flutter/material.dart';

import 'colored_wallet_theme.dart';

class DarkWalletTheme {
  DarkWalletTheme._();

  // ColorScheme
  static const colorScheme = ColorScheme.dark(
    brightness: Brightness.dark,
    primary: _Colors.primary,
    secondary: Color(0xFFA5C8FF),
    onSecondary: Color(0XFF383EDE),
    error: _Colors.textError,
    surface: _Colors.pageBackground,
    inverseSurface: Color(0xFF414966),
    primaryContainer: _Colors.pageContainers,
    onPrimaryContainer: _Colors.textPrimary,
    secondaryContainer: Color(0xFF004785),
    tertiaryContainer: _Colors.pageGutter,
    onPrimary: Color(0xFF002C71),
    onSurface: _Colors.textPrimary,
    onSurfaceVariant: Color(0xFF8292CC),
    outlineVariant: Color(0xFF656D87),
    shadow: Color(0x14FFFFFF),
  );

  static final theme = ColoredWalletTheme(
    const WalletThemeColors(
      colorScheme: colorScheme,
      textColor: _Colors.textPrimary,
      focusColor: Colors.white12,
      bottomNavigationUnselectedColor: Color(0xFFAAACB3),
      modalBackground: _Colors.modalBg,
      modalBarrier: _Colors.modalBarrierColor,
      actionPrimaryHover: _Colors.actionPrimaryHover,
      actionPrimaryBackgroundHover: _Colors.actionPrimaryBgHover,
      actionSecondary: _Colors.actionSecondary,
      elevatedButtonForeground: _Colors.textWhite,
    ),
  );

  static final textTheme = theme.textTheme;
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
  static const Color actionPrimaryBg = Colors.transparent;
  static const Color actionPrimaryHover = Color(0xFFC8D5FF);
  static const Color actionPrimaryBgHover = Color(0x1A8592B3);

  // Modals (dialogs & bottom sheets)
  static const Color modalBg = Color(0xFF292D34);
  static const Color modalBarrierColor = Color(0x80000000);

  // Pages
  static const Color pageOverlay = Color(0xFF1C1E25);
  static const Color pagePlaceholder = Color(0xFF616E99);
  static const Color pageContainers = Color(0xFF2F3444);
  static const Color pageGutter = Color(0xFF292D3A);
  static const Color pageBackground = Color(0xFF1C1E25);
  static const Color pageSpacer = Color(0xFF33343B);
}
