import 'package:flutter/widgets.dart';

extension ColorExtensions on Color {
  Color darken({double darkenAmount = 0.075}) {
    final hsl = HSLColor.fromColor(this);
    final lightness = (hsl.lightness - darkenAmount).clamp(0.0, 1.0);
    final hslDark = hsl.withLightness(lightness);
    return hslDark.toColor();
  }
}
