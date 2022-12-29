import 'dart:math';

import 'package:flutter/material.dart';

import 'keyboard_backspace_key.dart';
import 'keyboard_digit_key.dart';
import 'keyboard_row.dart';

const _preferredKeyboardHeight = 340.0;
const _maxHeightAsFractionOfScreen = 0.44;

class PinKeyboard extends StatelessWidget {
  final Function(int)? onKeyPressed;
  final VoidCallback? onBackspacePressed;

  const PinKeyboard({this.onKeyPressed, this.onBackspacePressed, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final maxHeight = MediaQuery.of(context).size.height * _maxHeightAsFractionOfScreen;
    return SizedBox(
      height: min(maxHeight, _preferredKeyboardHeight),
      child: DefaultTextStyle(
        style: Theme.of(context).textTheme.headline2!,
        child: Column(
          children: [
            KeyboardRow(
              children: [
                KeyboardDigitKey(digit: 1, onKeyPressed: onKeyPressed),
                KeyboardDigitKey(digit: 2, onKeyPressed: onKeyPressed),
                KeyboardDigitKey(digit: 3, onKeyPressed: onKeyPressed),
              ],
            ),
            KeyboardRow(
              children: [
                KeyboardDigitKey(digit: 4, onKeyPressed: onKeyPressed),
                KeyboardDigitKey(digit: 5, onKeyPressed: onKeyPressed),
                KeyboardDigitKey(digit: 6, onKeyPressed: onKeyPressed),
              ],
            ),
            KeyboardRow(
              children: [
                KeyboardDigitKey(digit: 7, onKeyPressed: onKeyPressed),
                KeyboardDigitKey(digit: 8, onKeyPressed: onKeyPressed),
                KeyboardDigitKey(digit: 9, onKeyPressed: onKeyPressed),
              ],
            ),
            KeyboardRow(
              children: [
                const Spacer(),
                KeyboardDigitKey(digit: 0, onKeyPressed: onKeyPressed),
                KeyboardBackspaceKey(onBackspacePressed: onBackspacePressed),
              ],
            )
          ],
        ),
      ),
    );
  }
}
