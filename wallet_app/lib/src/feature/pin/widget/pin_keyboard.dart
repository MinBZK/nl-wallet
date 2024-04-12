import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../util/extension/build_context_extension.dart';
import 'keyboard_backspace_key.dart';
import 'keyboard_digit_key.dart';
import 'keyboard_row.dart';

const _maxHeight = 340.0;
const _maxHeightAsFractionOfScreen = 0.44;
final _keyboardFocusNode = FocusNode(debugLabel: 'PinKeyboard');

class PinKeyboard extends StatelessWidget {
  final Function(int)? onKeyPressed;
  final VoidCallback? onBackspacePressed;
  final VoidCallback? onBackspaceLongPressed;

  /// The color used to draw the digits and backspace icon, defaults to [ColorScheme.onBackground]
  final Color? color;

  const PinKeyboard({
    this.onKeyPressed,
    this.onBackspacePressed,
    this.onBackspaceLongPressed,
    this.color,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final keyColor = color ?? context.colorScheme.onBackground;
    return KeyboardListener(
      key: const Key('pinKeyboard'),
      focusNode: _keyboardFocusNode,
      autofocus: true,
      onKeyEvent: _handleKeyEvent,
      child: ConstrainedBox(
        constraints: BoxConstraints(maxHeight: _maxKeyboardHeight(context)),
        child: DefaultTextStyle(
          style: context.textTheme.displayMedium!.copyWith(color: keyColor),
          child: Column(
            children: [
              KeyboardRow(
                children: [
                  KeyboardDigitKey(digit: 1, onKeyPressed: onKeyPressed, key: const Key('keyboardDigitKey#1')),
                  KeyboardDigitKey(digit: 2, onKeyPressed: onKeyPressed, key: const Key('keyboardDigitKey#2')),
                  KeyboardDigitKey(digit: 3, onKeyPressed: onKeyPressed, key: const Key('keyboardDigitKey#3')),
                ],
              ),
              KeyboardRow(
                children: [
                  KeyboardDigitKey(digit: 4, onKeyPressed: onKeyPressed, key: const Key('keyboardDigitKey#4')),
                  KeyboardDigitKey(digit: 5, onKeyPressed: onKeyPressed, key: const Key('keyboardDigitKey#5')),
                  KeyboardDigitKey(digit: 6, onKeyPressed: onKeyPressed, key: const Key('keyboardDigitKey#6')),
                ],
              ),
              KeyboardRow(
                children: [
                  KeyboardDigitKey(digit: 7, onKeyPressed: onKeyPressed, key: const Key('keyboardDigitKey#7')),
                  KeyboardDigitKey(digit: 8, onKeyPressed: onKeyPressed, key: const Key('keyboardDigitKey#8')),
                  KeyboardDigitKey(digit: 9, onKeyPressed: onKeyPressed, key: const Key('keyboardDigitKey#9')),
                ],
              ),
              KeyboardRow(
                children: [
                  const Spacer(),
                  KeyboardDigitKey(digit: 0, onKeyPressed: onKeyPressed, key: const Key('keyboardDigitKey#0')),
                  KeyboardBackspaceKey(
                    color: keyColor,
                    onBackspacePressed: onBackspacePressed,
                    onBackspaceLongPressed: onBackspaceLongPressed,
                    key: const Key('keyboardKeyBackspace'),
                  ),
                ],
              )
            ],
          ),
        ),
      ),
    );
  }

  void _handleKeyEvent(key) {
    if (key is! KeyDownEvent) return;
    final digit = int.tryParse(key.character ?? '');
    if (digit != null) {
      onKeyPressed?.call(digit);
    } else if (key.logicalKey == LogicalKeyboardKey.backspace) {
      onBackspacePressed?.call();
    }
  }

  double _maxKeyboardHeight(BuildContext context) {
    final mq = context.mediaQuery;
    if (mq.orientation == Orientation.portrait) {
      return min(_maxHeight, mq.size.height * _maxHeightAsFractionOfScreen);
    } else {
      return _maxHeight;
    }
  }
}
