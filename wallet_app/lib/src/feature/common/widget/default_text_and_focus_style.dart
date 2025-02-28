import 'package:flutter/material.dart';

import '../../../util/extension/text_style_extension.dart';

/// Use this widget to override the default text style (e.g. font size) of the child widget(s),
/// with the option to keep the same behavior (e.g. underline in focussed state) as defined in the theme.
class DefaultTextAndFocusStyle extends StatelessWidget {
  final Widget child;
  final WidgetStatesController statesController;
  final TextStyle? textStyle;
  final Color? pressedOrFocusedColor;
  final bool underlineWhenPressedOrFocused;

  const DefaultTextAndFocusStyle({
    super.key,
    required this.child,
    required this.statesController,
    required this.textStyle,
    this.pressedOrFocusedColor,
    this.underlineWhenPressedOrFocused = true,
  });

  @override
  Widget build(BuildContext context) {
    final localTextStyle = textStyle;
    if (localTextStyle == null) return child;

    // Build current text style with the correct color and underline state
    TextStyle currentTextStyle =
        localTextStyle.colorWhenPressedOrFocused(statesController.value, pressedOrFocusedColor);
    if (underlineWhenPressedOrFocused) {
      currentTextStyle = currentTextStyle.underlineWhenPressedOrFocused(statesController.value);
    }

    return DefaultTextStyle(
      style: currentTextStyle,
      child: child,
    );
  }
}
