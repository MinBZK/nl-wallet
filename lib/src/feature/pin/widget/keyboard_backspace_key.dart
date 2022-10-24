import 'package:flutter/material.dart';

class KeyboardBackspaceKey extends StatelessWidget {
  final VoidCallback? onBackspacePressed;

  const KeyboardBackspaceKey({this.onBackspacePressed, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: InkWell(
        onTap: onBackspacePressed == null ? null : () => onBackspacePressed!(),
        child: const Icon(Icons.backspace),
      ),
    );
  }
}
