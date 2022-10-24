import 'package:flutter/material.dart';

class KeyboardDigitKey extends StatelessWidget {
  final int digit;
  final Function(int)? onKeyPressed;

  const KeyboardDigitKey({required this.digit, this.onKeyPressed, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: InkWell(
        onTap: onKeyPressed == null ? null : () => onKeyPressed!(digit),
        child: Center(
          child: Text(
            digit.toString(),
            textAlign: TextAlign.center,
          ),
        ),
      ),
    );
  }
}
