import 'package:flutter/material.dart';

class WalletThemeConstants {
  WalletThemeConstants._();

  // Color scheme
  static const colorScheme = ColorScheme.light(
    primary: Color(0xFF2065E0),
    secondary: Color(0x332065E0),
    error: Color(0xFFCA005D),
    background: Color(0xFFFCFCFC),
    secondaryContainer: Color(0xFFF3F4F7),
    onPrimary: Color(0xFFFCFCFC),
    onBackground: Color(0xFF152A62),
    onSurface: Color(0xFF445581),
  );

  // Default font family
  static const fontFamily = 'RijksoverheidSansWebText';

  // Colors
  static const dividerColor = Color(0x66445581);
  static const indicatorColor = Color(0xFF2065E0);
  static const neutralDarkBlueColor = Color(0xFF0D193B);
  static const primaryColorDark = Color(0xFF152A62);
  static const scaffoldBackgroundColor = Color(0xFFFCFCFC);

  // App bar theme
  static final appBarTheme = AppBarTheme(
    backgroundColor: const Color(0xFFFCFCFC),
    centerTitle: true,
    elevation: 0.0,
    shape: const Border(bottom: BorderSide(color: Color(0xFFE8EAEF))),
    foregroundColor: const Color(0xDE000000),
    iconTheme: const IconThemeData(color: Color(0xFF152A62)),
    titleTextStyle: textTheme.subtitle1,
  );

  // Bottom navigation bar theme
  static const bottomNavigationBarThemeData = BottomNavigationBarThemeData(
    backgroundColor: Colors.white,
    elevation: 4.0,
    selectedLabelStyle: TextStyle(fontSize: 12, fontWeight: FontWeight.w700, fontFamily: fontFamily),
    unselectedLabelStyle: TextStyle(fontSize: 12, fontWeight: FontWeight.w400, fontFamily: fontFamily),
  );

  static final elevatedButtonTheme = ElevatedButtonThemeData(
    style: ElevatedButton.styleFrom(
      elevation: 0.0,
      minimumSize: const Size.fromHeight(_buttonMinHeight),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(_buttonBorderRadius)),
      foregroundColor: Colors.white,
      textStyle: _buttonTextStyle,
    ),
  );

  static const floatingActionButtonTheme = FloatingActionButtonThemeData(
    backgroundColor: Color(0xFF2065E0),
    foregroundColor: Colors.white,
    extendedTextStyle: _buttonTextStyle,
  );

  static final outlinedButtonTheme = OutlinedButtonThemeData(
    style: OutlinedButton.styleFrom(
      elevation: 0,
      minimumSize: const Size.fromHeight(_buttonMinHeight),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(_buttonBorderRadius)),
      textStyle: _buttonTextStyle,
      side: const BorderSide(color: Color(0xFF2065E0), width: 0.5),
    ),
  );

  static final textButtonTheme = TextButtonThemeData(
    style: TextButton.styleFrom(
      minimumSize: const Size(0.0, _buttonMinHeight),
      textStyle: _buttonTextStyle,
      foregroundColor: const Color(0xFF2065E0),
    ),
  );

  static const tabBarTheme = TabBarTheme(
    labelColor: Color(0xFF2065E0),
    labelStyle: _subtitle2TextStyle,
    unselectedLabelColor: Color(0xFF152A62),
    unselectedLabelStyle: _body2TextStyle,
  );

  static const scrollbarTheme = ScrollbarThemeData(
    thumbColor: MaterialStatePropertyAll(Color(0xFF152A62)),
    thickness: MaterialStatePropertyAll(4.0),
    crossAxisMargin: 8.0,
    mainAxisMargin: 8.0,
    radius: Radius.circular(8),
  );

  // Text theme
  static final textTheme = const TextTheme(
    headline1: _headline1TextStyle,
    headline2: _headline2TextStyle,
    headline3: _headline3TextStyle,
    headline4: _headline4TextStyle,
    subtitle1: _subtitle1TextStyle,
    subtitle2: _subtitle2TextStyle,
    bodyText1: _body1TextStyle,
    bodyText2: _body2TextStyle,
    button: _buttonTextStyle,
    caption: _captionTextStyle,
    overline: _overlineTextStyle,
  ).apply(
    bodyColor: const Color(0xFF152A62),
    displayColor: const Color(0xFF152A62),
    fontFamily: fontFamily,
  );

  // Button theme
  static const _buttonMinHeight = 48.0;
  static const _buttonBorderRadius = 8.0;

  // Text styles
  static const _headline1TextStyle = TextStyle(
    fontSize: _headline1FontSize,
    fontWeight: FontWeight.bold,
  );
  static const _headline2TextStyle = TextStyle(
    fontSize: _headline2FontSize,
    fontWeight: FontWeight.bold,
  );
  static const _headline3TextStyle = TextStyle(
    fontSize: _headline3FontSize,
    fontWeight: FontWeight.bold,
  );
  static const _headline4TextStyle = TextStyle(
    fontSize: _headline4FontSize,
    fontWeight: FontWeight.bold,
  );
  static const _subtitle1TextStyle = TextStyle(
    fontSize: _subtitle1FontSize,
    fontWeight: FontWeight.bold,
    height: 1.4,
  );
  static const _subtitle2TextStyle = TextStyle(
    fontSize: _subtitle2FontSize,
    fontWeight: FontWeight.bold,
  );
  static const _body1TextStyle = TextStyle(
    fontSize: _body1FontSize,
    height: 1.5,
  );
  static const _body2TextStyle = TextStyle(
    fontSize: _body2FontSize,
    height: 1.4,
  );
  static const _buttonTextStyle = TextStyle(
    fontSize: _buttonFontSize,
    fontWeight: FontWeight.bold,
  );
  static const _captionTextStyle = TextStyle(
    fontSize: _captionFontSize,
  );
  static const _overlineTextStyle = TextStyle(
    fontSize: _overlineFontSize,
    fontWeight: FontWeight.bold,
  );

  // Font sizes
  static const _headline1FontSize = 34.0;
  static const _headline2FontSize = 24.0;
  static const _headline3FontSize = 20.0;
  static const _headline4FontSize = 18.0;
  static const _subtitle1FontSize = 16.0;
  static const _subtitle2FontSize = 14.0;
  static const _body1FontSize = 16.0;
  static const _body2FontSize = 14.0;
  static const _buttonFontSize = 16.0;
  static const _captionFontSize = 12.0;
  static const _overlineFontSize = 14.0;
}
