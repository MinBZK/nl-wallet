import 'package:flutter/material.dart';

class TextIconButton extends StatelessWidget {
  final Widget child;
  final VoidCallback? onPressed;
  final IconPosition iconPosition;
  final IconData icon;

  /// Centers [child] inside full width of the [TextIconButton] widget.
  final bool centerChild;

  const TextIconButton({
    required this.child,
    required this.onPressed,
    this.icon = Icons.arrow_forward,
    this.iconPosition = IconPosition.end,
    this.centerChild = true,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final children = [
      if (centerChild) const SizedBox(width: 20),
      child,
      const SizedBox(width: 8),
      Icon(icon, size: 16),
    ];
    return TextButton(
      onPressed: onPressed,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: iconPosition == IconPosition.end ? children : children.reversed.toList(),
      ),
    );
  }
}

enum IconPosition { start, end }
