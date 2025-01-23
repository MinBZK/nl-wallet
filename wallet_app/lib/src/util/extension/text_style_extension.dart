import 'package:flutter/cupertino.dart';

import '../../theme/base_wallet_theme.dart';

extension TextStyleExtension on TextStyle {
  /// Adds underline decoration to given text style
  TextStyle get underlined => copyWith(decoration: TextDecoration.underline);

  /// Checks the current state of the [_statesController] and returns an altered color [TextStyle] when
  /// this widget is pressed or focused.
  TextStyle colorWhenPressedOrFocused(Set<WidgetState> states, Color pressedOrFocusedColor) {
    final property = WidgetStateProperty.resolveWith((states) {
      if (states.isPressedOrFocused) return copyWith(color: pressedOrFocusedColor);
      return this;
    });
    return property.resolve(states);
  }

  /// Checks the current state of the [_statesController] and returns a underlined [TextStyle] when
  /// this widget is pressed or focused.
  TextStyle underlineWhenPressedOrFocused(Set<WidgetState> states) {
    final property = WidgetStateProperty.resolveWith((states) {
      if (states.isPressedOrFocused) return underlined;
      return this;
    });
    return property.resolve(states);
  }
}
