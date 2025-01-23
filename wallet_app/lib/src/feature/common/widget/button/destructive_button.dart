import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/color_extension.dart';
import 'button_content.dart';

class DestructiveButton extends StatelessWidget {
  final VoidCallback? onPressed;
  final Text text;
  final Widget? icon;
  final IconPosition iconPosition;

  const DestructiveButton({
    this.onPressed,
    required this.text,
    this.icon = const Icon(Icons.arrow_forward_outlined),
    this.iconPosition = IconPosition.start,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final errorColor = context.colorScheme.error;
    final overlayColor = context.brightness == Brightness.light ? errorColor.darken() : errorColor.lighten();
    return ElevatedButton(
      style: context.theme.elevatedButtonTheme.style?.copyWith(
        backgroundColor: WidgetStateColor.resolveWith((states) => errorColor),
        overlayColor: WidgetStateColor.resolveWith((states) => overlayColor),
      ),
      onPressed: onPressed,
      child: ButtonContent(
        text: text,
        icon: icon,
        iconPosition: iconPosition,
      ),
    );
  }
}
