import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

// ignore: avoid_positional_boolean_parameters
typedef FocusWidgetBuilder = Widget Function(BuildContext context, bool hasFocus);

/// A widget that rebuilds its child based on whether it has keyboard focus.
///
/// `FocusBuilder` simplifies the process of creating UIs that react to focus changes.
/// It wraps Flutter's `Focus` widget and exposes the focus state to a `builder` function.
///
/// Additionally, it can optionally trigger a callback when the Enter key is pressed
/// while the widget has focus.
class FocusBuilder extends StatefulWidget {
  /// A builder function that constructs the UI based on whether the widget currently has focus.
  final FocusWidgetBuilder builder;

  /// An optional callback that is triggered when the enter key is pressed while this widget is focused.
  final VoidCallback? onEnterPressed;

  /// Whether this widget is allowed to request or receive focus.
  ///
  /// This is useful when you want to visually represent focus state without making
  /// the widget interactive via keyboard focus.
  final bool canRequestFocus;

  const FocusBuilder({
    required this.builder,
    this.onEnterPressed,
    this.canRequestFocus = true,
    super.key,
  });

  @override
  State<FocusBuilder> createState() => _FocusBuilderState();
}

class _FocusBuilderState extends State<FocusBuilder> {
  bool hasFocus = false;

  @override
  Widget build(BuildContext context) {
    return Focus(
      canRequestFocus: widget.canRequestFocus,
      onFocusChange: (focus) => setState(() => hasFocus = focus),
      onKeyEvent: widget.onEnterPressed == null
          ? null
          : (node, event) {
              final isKeyDownEvent = event is KeyDownEvent; // Only trigger once
              final isEnterEvent = event.logicalKey == LogicalKeyboardKey.enter;
              if (isKeyDownEvent && isEnterEvent) {
                widget.onEnterPressed!();
                return KeyEventResult.handled;
              }
              return KeyEventResult.ignored;
            },
      child: widget.builder(context, hasFocus),
    );
  }
}
