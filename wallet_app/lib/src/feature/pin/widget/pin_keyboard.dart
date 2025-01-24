import 'dart:async';
import 'dart:math';

import 'package:after_layout/after_layout.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../../util/extension/build_context_extension.dart';
import 'keyboard_backspace_key.dart';
import 'keyboard_biometric_key.dart';
import 'keyboard_digit_key.dart';
import 'keyboard_row.dart';

const _maxHeight = 340.0;
const _maxHeightAsFractionOfScreen = 0.44;

class PinKeyboard extends StatefulWidget {
  final Function(int)? onKeyPressed;
  final VoidCallback? onBackspacePressed;
  final VoidCallback? onBackspaceLongPressed;

  /// Called when the user presses the 'Biometrics' key. This key is only visible when the callback is provided.
  final VoidCallback? onBiometricsPressed;

  bool get showBiometricsButton => onBiometricsPressed != null;

  const PinKeyboard({
    this.onKeyPressed,
    this.onBackspacePressed,
    this.onBackspaceLongPressed,
    this.onBiometricsPressed,
    super.key,
  });

  @override
  State<PinKeyboard> createState() => _PinKeyboardState();
}

class _PinKeyboardState extends State<PinKeyboard> with AfterLayoutMixin<PinKeyboard> {
  late FocusNode keyboardFocusNode;

  @override
  void initState() {
    super.initState();
    keyboardFocusNode = FocusNode(skipTraversal: true);
  }

  @override
  FutureOr<void> afterFirstLayout(BuildContext context) {
    keyboardFocusNode.requestFocus();
  }

  @override
  void dispose() {
    keyboardFocusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return KeyboardListener(
      focusNode: keyboardFocusNode,
      autofocus: true,
      onKeyEvent: _handleKeyEvent,
      child: ConstrainedBox(
        constraints: BoxConstraints(maxHeight: _maxKeyboardHeight(context)),
        child: Column(
          children: [
            KeyboardRow(
              children: [
                KeyboardDigitKey(digit: 1, onKeyPressed: widget.onKeyPressed, key: const Key('keyboardDigitKey#1')),
                KeyboardDigitKey(digit: 2, onKeyPressed: widget.onKeyPressed, key: const Key('keyboardDigitKey#2')),
                KeyboardDigitKey(digit: 3, onKeyPressed: widget.onKeyPressed, key: const Key('keyboardDigitKey#3')),
              ],
            ),
            KeyboardRow(
              children: [
                KeyboardDigitKey(digit: 4, onKeyPressed: widget.onKeyPressed, key: const Key('keyboardDigitKey#4')),
                KeyboardDigitKey(digit: 5, onKeyPressed: widget.onKeyPressed, key: const Key('keyboardDigitKey#5')),
                KeyboardDigitKey(digit: 6, onKeyPressed: widget.onKeyPressed, key: const Key('keyboardDigitKey#6')),
              ],
            ),
            KeyboardRow(
              children: [
                KeyboardDigitKey(digit: 7, onKeyPressed: widget.onKeyPressed, key: const Key('keyboardDigitKey#7')),
                KeyboardDigitKey(digit: 8, onKeyPressed: widget.onKeyPressed, key: const Key('keyboardDigitKey#8')),
                KeyboardDigitKey(digit: 9, onKeyPressed: widget.onKeyPressed, key: const Key('keyboardDigitKey#9')),
              ],
            ),
            KeyboardRow(
              children: [
                widget.showBiometricsButton
                    ? KeyboardBiometricKey(onPressed: widget.onBiometricsPressed)
                    : const Spacer(),
                KeyboardDigitKey(digit: 0, onKeyPressed: widget.onKeyPressed, key: const Key('keyboardDigitKey#0')),
                KeyboardBackspaceKey(
                  onBackspacePressed: widget.onBackspacePressed,
                  onBackspaceLongPressed: widget.onBackspaceLongPressed,
                  key: const Key('keyboardKeyBackspace'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  void _handleKeyEvent(key) {
    if (key is! KeyDownEvent) return;
    final digit = int.tryParse(key.character ?? '');
    if (digit != null) {
      widget.onKeyPressed?.call(digit);
    } else if (key.logicalKey == LogicalKeyboardKey.backspace) {
      widget.onBackspacePressed?.call();
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
