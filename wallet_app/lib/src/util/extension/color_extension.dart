import 'package:flutter/widgets.dart';

extension ColorExtensions on Color {
  Color darken({double darkenAmount = 0.075}) {
    final hsl = HSLColor.fromColor(this);
    final lightness = (hsl.lightness - darkenAmount).clamp(0.0, 1.0);
    final hslDark = hsl.withLightness(lightness);
    return hslDark.toColor();
  }

  Color lighten({double lightenAmount = 0.075}) {
    final hsl = HSLColor.fromColor(this);
    final lightness = (hsl.lightness + lightenAmount).clamp(0.0, 1.0);
    final hslLight = hsl.withLightness(lightness);
    return hslLight.toColor();
  }
}
