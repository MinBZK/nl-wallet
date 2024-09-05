import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

// ignore: avoid_positional_boolean_parameters
typedef FocusWidgetBuilder = Widget Function(BuildContext context, bool hasFocus);

class FocusBuilder extends StatefulWidget {
  final FocusWidgetBuilder builder;
  final VoidCallback? onEnterPressed;

  const FocusBuilder({
    required this.builder,
    this.onEnterPressed,
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
