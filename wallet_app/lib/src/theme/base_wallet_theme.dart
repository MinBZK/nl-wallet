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
    fontWeight: FontWeight.bold,
    height: 26 / 18,
    letterSpacing: 0.15,
    fontFamily: fontFamily,
  );

  static final baseTextTheme = const TextTheme(
    // DISPLAY
    displayLarge: TextStyle(fontSize: 56, fontWeight: FontWeight.normal, height: 84 / 56, letterSpacing: 0.25),
    displayMedium: TextStyle(fontSize: 44, fontWeight: FontWeight.normal, height: 66 / 44, letterSpacing: 0),
    displaySmall: TextStyle(fontSize: 36, fontWeight: FontWeight.normal, height: 54 / 36, letterSpacing: 0),
    // HEADLINE
    headlineLarge: TextStyle(fontSize: 30, fontWeight: FontWeight.bold, height: 44 / 30, letterSpacing: 0),
    headlineMedium: TextStyle(fontSize: 24, fontWeight: FontWeight.bold, height: 36 / 24, letterSpacing: 0.15),
    headlineSmall: TextStyle(fontSize: 20, fontWeight: FontWeight.bold, height: 30 / 20, letterSpacing: 0.15),
    // TITLE
    titleLarge: TextStyle(fontSize: 18, fontWeight: FontWeight.bold, height: 26 / 18, letterSpacing: 0.15),
    titleMedium: TextStyle(fontSize: 16, fontWeight: FontWeight.bold, height: 24 / 16, letterSpacing: 0.15),
    titleSmall: TextStyle(fontSize: 14, fontWeight: FontWeight.bold, height: 20 / 14, letterSpacing: 0.15),
    // LABEL
    labelLarge: TextStyle(fontSize: 16, fontWeight: FontWeight.bold, height: 20 / 16, letterSpacing: 1),
    labelMedium: TextStyle(fontSize: 14, fontWeight: FontWeight.bold, height: 20 / 14, letterSpacing: 1),
    labelSmall: TextStyle(fontSize: 12, fontWeight: FontWeight.bold, height: 18 / 12, letterSpacing: 1),
    // BODY
    bodyLarge: TextStyle(fontSize: 16, fontWeight: FontWeight.normal, height: 24 / 16, letterSpacing: 0.5),
    bodyMedium: TextStyle(fontSize: 14, fontWeight: FontWeight.normal, height: 20 / 14, letterSpacing: 0.25),
    bodySmall: TextStyle(fontSize: 12, fontWeight: FontWeight.normal, height: 18 / 12, letterSpacing: 0.15),
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
    titleTextStyle: baseTextTheme.headlineMedium!.copyWith(fontWeight: FontWeight.bold),
    scrolledUnderElevation: 12,
    shape: LinearBorder.none, /* hides the app bar divider */
  );

  static final baseTabBarTheme = TabBarTheme(
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
