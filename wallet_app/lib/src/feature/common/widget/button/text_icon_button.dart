import 'package:flutter/material.dart';

class TextIconButton extends StatelessWidget {
  final Widget child;
  final VoidCallback? onPressed;
  final IconPosition iconPosition;
  final IconData icon;
  final Alignment contentAlignment;

  /// Centers [child] inside full width of the [TextIconButton] widget.
  final bool centerChild;

  const TextIconButton({
    required this.child,
    required this.onPressed,
    this.icon = Icons.arrow_forward,
    this.iconPosition = IconPosition.end,
    this.centerChild = true,
    this.contentAlignment = Alignment.center,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final children = [
      if (centerChild) const SizedBox(width: 20),
      Flexible(child: child),
      const SizedBox(width: 8),
      Icon(icon, size: 16),
    ];
    return TextButton(
      onPressed: onPressed,
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 8),
        child: Align(
          alignment: contentAlignment,
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: iconPosition == IconPosition.end ? children : children.reversed.toList(),
          ),
        ),
      ),
    );
  }
}

enum IconPosition { start, end }
