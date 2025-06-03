import 'package:flutter/material.dart';

import '../util/extension/text_style_extension.dart';

/// Base Wallet Theme
///
/// Dark / Light classes of the app specify the dedicated colors, but items like textStyles and
/// radii, which are common across the [LightWalletTheme] and [DarkWalletTheme] are specified
/// here as baseThemes, intended to be extended with the correct colors later.

class BaseWalletTheme {
  BaseWalletTheme._();

  //region Font & TextStyles
  static const fontFamily = 'RijksoverheidSansWebText';

  static const headlineExtraSmallTextStyle = TextStyle(
    fontSize: 18,
    fontVariations: [fontVariationBold],
    height: 24 / 18,
    letterSpacing: 0,
    fontFamily: fontFamily,
  );

  static const fontVariationRegular = FontVariation('wght', 400);
  static const fontVariationBold = FontVariation('wght', 700);

  static final baseTextTheme = const TextTheme(
    // DISPLAY
    displayLarge: TextStyle(
      fontSize: 56,
      height: 78 / 56,
      letterSpacing: 0,
      fontVariations: [fontVariationRegular],
    ),
    displayMedium: TextStyle(
      fontSize: 44,
      height: 62 / 44,
      letterSpacing: 0,
      fontVariations: [fontVariationRegular],
    ),
    displaySmall: TextStyle(
      fontSize: 36,
      height: 50 / 36,
      letterSpacing: 0,
      fontVariations: [fontVariationRegular],
    ),
    // HEADLINE
    headlineLarge: TextStyle(
      fontSize: 30,
      height: 42 / 30,
      letterSpacing: 0,
      fontVariations: [fontVariationBold],
    ),
    headlineMedium: TextStyle(
      fontSize: 24,
      height: 34 / 24,
      letterSpacing: 0,
      fontVariations: [fontVariationBold],
    ),
    headlineSmall: TextStyle(
      fontSize: 20,
      height: 28 / 20,
      letterSpacing: 0,
      fontVariations: [fontVariationBold],
    ),
    // TITLE
    titleLarge: TextStyle(
      fontSize: 18,
      height: 24 / 18,
      letterSpacing: 0,
      fontVariations: [fontVariationBold],
    ),
    titleMedium: TextStyle(
      fontSize: 16,
      height: 22 / 16,
      letterSpacing: 0,
      fontVariations: [fontVariationBold],
    ),
    titleSmall: TextStyle(
      fontSize: 14,
      height: 20 / 14,
      letterSpacing: 0,
      fontVariations: [fontVariationBold],
    ),
    // LABEL
    labelLarge: TextStyle(
      fontSize: 18,
      height: 24 / 18,
      letterSpacing: 0.5,
      fontVariations: [fontVariationBold],
    ),
    labelMedium: TextStyle(
      fontSize: 16,
      height: 22 / 16,
      letterSpacing: 0.5,
      fontVariations: [fontVariationBold],
    ),
    labelSmall: TextStyle(
      fontSize: 14,
      height: 20 / 14,
      letterSpacing: 0.5,
      fontVariations: [fontVariationBold],
    ),
    // BODY
    bodyLarge: TextStyle(
      fontSize: 16,
      height: 22 / 16,
      letterSpacing: 0,
      fontVariations: [fontVariationRegular],
    ),
    bodyMedium: TextStyle(
      fontSize: 14,
      height: 20 / 14,
      letterSpacing: 0,
      fontVariations: [fontVariationRegular],
    ),
    bodySmall: TextStyle(
      fontSize: 12,
      height: 16 / 12,
      letterSpacing: 0,
      fontVariations: [fontVariationRegular],
    ),
  ).apply(fontFamily: fontFamily);

  //endregion Font & TextStyles

  //region Button Style & Themes
  static const _buttonBorderRadius = 12.0;
  static const _buttonMinHeight = 48.0;
  static const _buttonBorderWidthFocused = 2.0;
  static const _outlineButtonBorderWidthDefault = 0.5;
  static final _buttonShape = RoundedRectangleBorder(borderRadius: BorderRadius.circular(_buttonBorderRadius));

  static const buttonBorderSideFocused = BorderSide(
    strokeAlign: BorderSide.strokeAlignInside,
    width: _buttonBorderWidthFocused,
  );

  static const outlineButtonBorderSideDefault = BorderSide(
    width: _outlineButtonBorderWidthDefault,
  );

  static final _baseButtonStyleIconSize = WidgetStateProperty.resolveWith(
    (states) => states.isPressedOrFocused ? 20.0 : 16.0,
  );

  static final _baseIconButtonStyleIconSize = WidgetStateProperty.resolveWith(
    (states) => states.isPressedOrFocused ? 30.0 : 24.0,
  );

  static final _defaultButtonTextStyle = WidgetStateTextStyle.resolveWith((states) {
    final textTheme = baseTextTheme.labelLarge!;
    return states.isPressedOrFocused ? textTheme.underlined : textTheme;
  });

  static final baseElevatedButtonTheme = ElevatedButtonThemeData(
    style: ElevatedButton.styleFrom(
      elevation: 0,
      minimumSize: const Size.fromHeight(_buttonMinHeight),
      padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 16),
      shape: _buttonShape,
    ).copyWith(
      iconSize: _baseButtonStyleIconSize,
      textStyle: _defaultButtonTextStyle,
    ),
  );

  static final baseOutlinedButtonTheme = OutlinedButtonThemeData(
    style: OutlinedButton.styleFrom(
      elevation: 0,
      minimumSize: const Size.fromHeight(_buttonMinHeight),
      padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 16),
      shape: _buttonShape,
    ).copyWith(
      iconSize: _baseButtonStyleIconSize,
      textStyle: _defaultButtonTextStyle,
    ),
  );

  static final baseTextButtonTheme = TextButtonThemeData(
    style: TextButton.styleFrom(
      minimumSize: const Size(0, _buttonMinHeight),
      padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 16),
      shape: _buttonShape,
    ).copyWith(
      iconSize: _baseButtonStyleIconSize,
      textStyle: _defaultButtonTextStyle,
    ),
  );

  static final baseIconButtonTheme = IconButtonThemeData(
    style: ButtonStyle(
      iconSize: _baseIconButtonStyleIconSize,
    ),
  );

  static final floatingActionButtonTheme = FloatingActionButtonThemeData(
    extendedTextStyle: baseTextTheme.labelLarge,
    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(50)),
  );

  //endregion Button Style & Themes

  //region Other Themes
  static const baseDividerTheme = DividerThemeData(space: 1, thickness: 1);

  static const baseBottomSheetTheme = BottomSheetThemeData(shape: ContinuousRectangleBorder());

  static final baseBottomNavigationBarThemeData = BottomNavigationBarThemeData(
    elevation: 0,
    selectedLabelStyle: baseTextTheme.labelSmall,
    unselectedLabelStyle: baseTextTheme.bodySmall,
  );

  static final baseAppBarTheme = AppBarTheme(
    centerTitle: false,
    elevation: 0,
    titleTextStyle: baseTextTheme.headlineMedium,
    scrolledUnderElevation: 12,
    shape: LinearBorder.none, /* hides the app bar divider */
  );

  static final baseTabBarTheme = TabBarThemeData(
    labelStyle: baseTextTheme.titleSmall,
    unselectedLabelStyle: baseTextTheme.bodyMedium,
    indicatorSize: TabBarIndicatorSize.tab,
  );

  /// Also see
  static const baseScrollbarTheme = ScrollbarThemeData(
    crossAxisMargin: 6,
    mainAxisMargin: 6,
    radius: Radius.zero,
    thickness: WidgetStatePropertyAll(4),
    thumbVisibility: WidgetStatePropertyAll(true),
    trackVisibility: WidgetStatePropertyAll(false),
  );

  static const baseIconTheme = IconThemeData(size: 16);

//endregion Other Themes
}

extension WidgetStateExtensions on Set<WidgetState> {
  bool get isPressedOrFocused => contains(WidgetState.pressed) || contains(WidgetState.focused);

  bool get isFocused => contains(WidgetState.focused);

  bool get isPressed => contains(WidgetState.pressed);
}
