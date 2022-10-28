import 'package:flutter/material.dart';

class TextArrowButton extends StatelessWidget {
  final Widget child;
  final VoidCallback? onPressed;
  final ArrowPosition arrowPosition;

  const TextArrowButton({
    required this.child,
    required this.onPressed,
    this.arrowPosition = ArrowPosition.end,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final children = [
      const SizedBox(width: 20),
      child,
      const SizedBox(width: 8),
      const Icon(Icons.arrow_forward, size: 12),
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
