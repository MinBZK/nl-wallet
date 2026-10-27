import 'package:flutter/material.dart';

import 'base_wallet_theme.dart';

/// The colors that differ between the [LightWalletTheme] and [DarkWalletTheme], consumed by [ColoredWalletTheme].
///
/// Colors that can be derived from the [colorScheme] are intentionally not part of this class.
class WalletThemeColors {
  final ColorScheme colorScheme;
  final Color textColor;
  final Color focusColor;
  final Color bottomNavigationUnselectedColor;
  final Color modalBackground;
  final Color modalBarrier;
  final Color actionPrimaryHover;
  final Color actionPrimaryBackgroundHover;
  final Color actionSecondary;
  final Color elevatedButtonForeground;

  const WalletThemeColors({
    required this.colorScheme,
    required this.textColor,
    required this.focusColor,
    required this.bottomNavigationUnselectedColor,
    required this.modalBackground,
    required this.modalBarrier,
    required this.actionPrimaryHover,
    required this.actionPrimaryBackgroundHover,
    required this.actionSecondary,
    required this.elevatedButtonForeground,
  });
}

/// Applies a [WalletThemeColors] palette on top of the [BaseWalletTheme] component themes.
///
/// This holds the structure that is shared between the [LightWalletTheme] and [DarkWalletTheme],
/// those classes only provide the palette.
class ColoredWalletTheme {
  final WalletThemeColors colors;

  ColoredWalletTheme(this.colors);

  ColorScheme get _colorScheme => colors.colorScheme;

  /// Returns a copy of [base] with the colored component themes applied.
  ThemeData applyTo(ThemeData base) => base.copyWith(
    appBarTheme: appBarTheme,
    bottomNavigationBarTheme: bottomNavigationBarTheme,
    bottomSheetTheme: bottomSheetTheme,
    brightness: _colorScheme.brightness,
    cardColor: _colorScheme.surface,
    colorScheme: _colorScheme,
    dialogTheme: dialogTheme,
    dividerTheme: dividerTheme,
    elevatedButtonTheme: elevatedButtonTheme,
    focusColor: colors.focusColor,
    iconButtonTheme: iconButtonTheme,
    iconTheme: iconTheme,
    outlinedButtonTheme: outlinedButtonTheme,
    primaryColorDark: colors.textColor,
    progressIndicatorTheme: progressIndicatorTheme,
    scaffoldBackgroundColor: _colorScheme.surface,
    scrollbarTheme: scrollBarTheme,
    switchTheme: switchTheme,
    tabBarTheme: tabBarTheme,
    textButtonTheme: textButtonTheme,
    textTheme: textTheme,
  );

  late final TextTheme textTheme = BaseWalletTheme.baseTextTheme.apply(
    bodyColor: colors.textColor,
    displayColor: colors.textColor,
  );

  late final DialogThemeData dialogTheme = BaseWalletTheme.baseDialogTheme.copyWith(
    backgroundColor: colors.modalBackground,
    barrierColor: colors.modalBarrier,
    titleTextStyle: textTheme.headlineMedium,
  );

  late final DividerThemeData dividerTheme = BaseWalletTheme.baseDividerTheme.copyWith(
    color: _colorScheme.outlineVariant,
  );

  late final AppBarTheme appBarTheme = BaseWalletTheme.baseAppBarTheme.copyWith(
    backgroundColor: _colorScheme.surface,
    surfaceTintColor: _colorScheme.surface,
    titleTextStyle: textTheme.headlineMedium,
    shadowColor: _colorScheme.shadow,
  );

  late final BottomNavigationBarThemeData bottomNavigationBarTheme = BaseWalletTheme.baseBottomNavigationBarThemeData
      .copyWith(
        backgroundColor: _colorScheme.surface,
        unselectedItemColor: colors.bottomNavigationUnselectedColor,
      );

  late final ElevatedButtonThemeData elevatedButtonTheme = ElevatedButtonThemeData(
    style: BaseWalletTheme.baseElevatedButtonTheme.style?.copyWith(
      backgroundColor: WidgetStateColor.resolveWith(
        (states) => states.isPressedOrFocused ? colors.actionPrimaryHover : _colorScheme.primary,
      ),
      foregroundColor: WidgetStatePropertyAll(colors.elevatedButtonForeground),
      iconColor: WidgetStatePropertyAll(colors.elevatedButtonForeground),
      overlayColor: WidgetStateProperty.resolveWith(
        (states) => states.isPressedOrFocused ? colors.actionPrimaryHover : null,
      ),
      side: WidgetStateBorderSide.resolveWith(
        (states) => states.isFocused ? _buttonBorderSideFocused : null,
      ),
    ),
  );

  late final OutlinedButtonThemeData outlinedButtonTheme = OutlinedButtonThemeData(
    style: BaseWalletTheme.baseOutlinedButtonTheme.style?.copyWith(
      backgroundColor: _actionBackground,
      foregroundColor: _actionForeground,
      iconColor: _actionForeground,
      overlayColor: _actionOverlay,
      side: WidgetStateBorderSide.resolveWith(
        (states) {
          if (states.isPressed) return _outlineButtonBorderSidePressed;
          if (states.isFocused) return _buttonBorderSideFocused;
          return _outlineButtonBorderSideDefault;
        },
      ),
    ),
  );

  late final TextButtonThemeData textButtonTheme = TextButtonThemeData(
    style: BaseWalletTheme.baseTextButtonTheme.style?.copyWith(
      backgroundColor: _actionBackground,
      foregroundColor: _actionForeground,
      iconColor: _actionForeground,
      overlayColor: _actionOverlay,
      side: WidgetStateBorderSide.resolveWith(
        (states) => states.isFocused ? _buttonBorderSideFocused : null,
      ),
    ),
  );

  late final IconButtonThemeData iconButtonTheme = IconButtonThemeData(
    style: BaseWalletTheme.baseIconButtonTheme.style?.copyWith(
      foregroundColor: _actionForeground,
      iconColor: _actionForeground,
      side: WidgetStateProperty.resolveWith(
        (states) => states.isFocused ? _buttonBorderSideFocused : null,
      ),
    ),
  );

  late final TabBarThemeData tabBarTheme = BaseWalletTheme.baseTabBarTheme.copyWith(
    labelColor: _colorScheme.primary,
    unselectedLabelColor: _colorScheme.onSurface,
    indicatorColor: _colorScheme.primary,
  );

  late final ScrollbarThemeData scrollBarTheme = BaseWalletTheme.baseScrollbarTheme.copyWith(
    thumbColor: WidgetStatePropertyAll(colors.textColor),
  );

  late final BottomSheetThemeData bottomSheetTheme = BaseWalletTheme.baseBottomSheetTheme.copyWith(
    backgroundColor: colors.modalBackground,
    modalBarrierColor: colors.modalBarrier,
  );

  late final IconThemeData iconTheme = BaseWalletTheme.baseIconTheme.copyWith(color: _colorScheme.onSurface);

  late final SwitchThemeData switchTheme = BaseWalletTheme.baseSwitchTheme.copyWith(
    trackOutlineColor: WidgetStateMapper({
      WidgetState.selected: _colorScheme.primary,
      WidgetState.any: colors.actionSecondary,
    }),
    thumbColor: WidgetStateMapper({
      WidgetState.selected: _colorScheme.surface,
      WidgetState.any: colors.actionSecondary,
    }),
  );

  late final ProgressIndicatorThemeData progressIndicatorTheme = ProgressIndicatorThemeData(
    linearTrackColor: _colorScheme.tertiaryContainer,
  );

  //region Shared button state properties
  late final WidgetStateColor _actionBackground = WidgetStateColor.resolveWith(
    (states) => states.isPressedOrFocused ? colors.actionPrimaryBackgroundHover : Colors.transparent,
  );

  late final WidgetStateColor _actionForeground = WidgetStateColor.resolveWith(
    (states) => states.isPressedOrFocused ? colors.actionPrimaryHover : _colorScheme.primary,
  );

  late final WidgetStateProperty<Color?> _actionOverlay = WidgetStateProperty.resolveWith(
    (states) => states.isPressedOrFocused ? colors.actionPrimaryBackgroundHover : null,
  );

  //endregion Shared button state properties

  //region Button border sides
  late final BorderSide _buttonBorderSideFocused = BaseWalletTheme.buttonBorderSideFocused.copyWith(
    color: colors.actionPrimaryHover,
  );

  late final BorderSide _outlineButtonBorderSideDefault = BaseWalletTheme.outlineButtonBorderSideDefault.copyWith(
    color: _colorScheme.primary,
  );

  late final BorderSide _outlineButtonBorderSidePressed = BaseWalletTheme.outlineButtonBorderSideDefault.copyWith(
    color: colors.actionPrimaryHover,
  );

  //endregion Button border sides
}
