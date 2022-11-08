import 'package:flutter/material.dart';

class TextIconButton extends StatelessWidget {
  final Widget child;
  final VoidCallback? onPressed;
  final ArrowPosition arrowPosition;
  final IconData icon;

  const TextIconButton({
    required this.child,
    required this.onPressed,
    this.icon = Icons.arrow_forward,
    this.arrowPosition = ArrowPosition.end,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final children = [
      const SizedBox(width: 20),
      child,
      const SizedBox(width: 8),
      Icon(icon, size: 12),
    ];
    return TextButton(
      onPressed: onPressed,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: arrowPosition == ArrowPosition.end ? children : children.reversed.toList(),
      ),
    );
  }
}

enum ArrowPosition { start, end }
