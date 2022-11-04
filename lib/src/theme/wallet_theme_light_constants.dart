import 'package:flutter/material.dart';

class WalletThemeConstants {
  WalletThemeConstants._();

  // Color scheme
  static const colorScheme = ColorScheme.light(
    primary: _primaryColor,
    error: _errorColor,
    secondaryContainer: _secondaryContainer,
  );

  // Indicator color
  static const indicatorColor = _primaryColor;

  // Default font family
  static const fontFamily = _defaultFontFamily;

  // Divider color
  static const dividerColor = _dividerColor;

  // Scaffold colors
  static const scaffoldBackgroundColor = _scaffoldBackgroundColor;

  // App bar theme
  static final appBarTheme = AppBarTheme(
    backgroundColor: _appBarBackgroundColor,
    centerTitle: _appBarCenterTitle,
    elevation: _appBarElevation,
    foregroundColor: _appBarForegroundColor,
    iconTheme: _appBarIconThemeData,
    titleTextStyle: _subtitle1TextStyle.copyWith(color: _primaryTextColor),
  );

  // Bottom navigation bar theme
  static const bottomNavigationBarThemeData = BottomNavigationBarThemeData(
    backgroundColor: _bottomNavigationBarBackgroundColor,
    elevation: _bottomNavigationBarElevation,
  );

  static final elevatedButtonTheme = ElevatedButtonThemeData(
    style: ElevatedButton.styleFrom(
      elevation: _elevatedButtonElevation,
      minimumSize: const Size.fromHeight(_elevatedButtonMinHeight),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(_elevatedButtonBorderRadius)),
      textStyle: _buttonTextStyle,
    ),
  );

  static const floatingActionButtonTheme = FloatingActionButtonThemeData(
    backgroundColor: _primaryColor,
    foregroundColor: Colors.white,
    extendedTextStyle: _buttonTextStyle,
  );

  static final outlinedButtonTheme = OutlinedButtonThemeData(
    style: OutlinedButton.styleFrom(
      elevation: 0,
      minimumSize: const Size.fromHeight(_outlinedButtonMinHeight),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(_outlinedButtonBorderRadius)),
      textStyle: _buttonTextStyle,
      side: const BorderSide(color: _primaryColor, width: 0.5),
    ),
  );

  static final textButtonTheme = TextButtonThemeData(
    style: TextButton.styleFrom(
      minimumSize: const Size(0.0, _textButtonMinHeight),
      textStyle: _buttonTextStyle,
      foregroundColor: _primaryColor,
    ),
  );

  static const tabBarTheme = TabBarTheme(
    labelColor: _primaryColor,
    labelStyle: _subtitle2TextStyle,
    unselectedLabelColor: _primaryTextColor,
    unselectedLabelStyle: _body2TextStyle,
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
    bodyColor: _primaryTextColor,
    displayColor: _primaryTextColor,
  );

  // Color scheme
  static const _primaryColor = Color(0xFF2065E0);
  static const _errorColor = Color(0xFFCA005D);
  static const _secondaryContainer = Color(0xFFF3F4F7);
  static const _defaultBackgroundColor = Color(0xFFFCFCFC);

  // App, bottom navigation bar & scaffold theme
  static const _appBarBackgroundColor = _defaultBackgroundColor;
  static const _appBarForegroundColor = Color(0xDE000000);
  static const _appBarCenterTitle = true;
  static const _appBarElevation = 1.0;
  static const _appBarIconThemeData = IconThemeData(color: _primaryTextColor);
  static const _bottomNavigationBarBackgroundColor = Colors.white;
  static const _bottomNavigationBarElevation = 4.0;
  static const _scaffoldBackgroundColor = _defaultBackgroundColor;
  static const _dividerColor = Color(0x66445581);

  // Button theme
  static const _elevatedButtonBorderRadius = 8.0;
  static const _elevatedButtonElevation = 0.0;
  static const _elevatedButtonMinHeight = 48.0;
  static const _outlinedButtonBorderRadius = _elevatedButtonBorderRadius;
  static const _outlinedButtonMinHeight = _elevatedButtonMinHeight;
  static const _textButtonMinHeight = _elevatedButtonMinHeight;

  // Text colors
  static const _primaryTextColor = Color(0xDE152A62);

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
    height: _subtitle1LineHeight,
  );
  static const _subtitle2TextStyle = TextStyle(
    fontSize: _subtitle2FontSize,
    fontWeight: FontWeight.bold,
  );
  static const _body1TextStyle = TextStyle(
    fontSize: _body1FontSize,
  );
  static const _body2TextStyle = TextStyle(
    fontSize: _body2FontSize,
    height: _body2LineHeight,
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

  // Font families
  static const _defaultFontFamily = 'RijksoverheidSansWebText';

  // Font sizes
  static const _headline1FontSize = 34.0;
  static const _headline2FontSize = 24.0;
  static const _headline3FontSize = 20.0;
  static const _headline4FontSize = 18.0;
  static const _subtitle1FontSize = 16.0;
  static const _subtitle2FontSize = 14.0;
  static const _body1FontSize = 16.0;
  static const _body2FontSize = 14.0;
  static const _buttonFontSize = 14.0;
  static const _captionFontSize = 12.0;
  static const _overlineFontSize = 10.0;

  // Line height
  static const _subtitle1LineHeight = 1.4;
  static const _body2LineHeight = 1.4;
}
