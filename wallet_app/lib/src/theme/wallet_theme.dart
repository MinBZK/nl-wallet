import 'package:flutter/material.dart';

import 'base_wallet_theme.dart';
import 'dark_wallet_theme.dart';
import 'light_wallet_theme.dart';

class WalletTheme {
  const WalletTheme._();

  static const kBorderRadius12 = BorderRadius.all(Radius.circular(12));

  static final ThemeData _baseTheme = ThemeData(
    useMaterial3: true,
    fontFamily: BaseWalletTheme.fontFamily,
    floatingActionButtonTheme: BaseWalletTheme.floatingActionButtonTheme,
    sliderTheme: const SliderThemeData(
      inactiveTrackColor: Color(0xFFABABAB),
      activeTrackColor: Colors.white,
      thumbColor: Colors.white,
      trackHeight: 4,
      thumbShape: RoundSliderThumbShape(enabledThumbRadius: 24 / 2),
      overlayShape: RoundSliderOverlayShape(overlayRadius: 50 / 2),
      overlayColor: Color(0xE61C1E25),
      padding: EdgeInsets.symmetric(horizontal: 2 /* provide tiny spacing for focused state */),
    ),
  );

  static final ThemeData light = LightWalletTheme.theme.applyTo(_baseTheme);

  static final ThemeData dark = DarkWalletTheme.theme.applyTo(_baseTheme);
}
