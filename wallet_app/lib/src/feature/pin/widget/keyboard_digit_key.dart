import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';

class KeyboardDigitKey extends StatelessWidget {
  final int digit;
  final Function(int)? onKeyPressed;

  const KeyboardDigitKey({required this.digit, this.onKeyPressed, super.key});

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: TextButton(
        onPressed: onKeyPressed == null ? null : () => onKeyPressed!(digit),
        style: context.theme.textButtonTheme.style?.copyWith(
          shape: WidgetStateProperty.all(
            const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
          ),
        ),
        child: Center(
          child: Semantics(
            keyboardKey: true,
            button: true,
            onTapHint: context.l10n.pinKeyboardWCAGDigitKeyTapHint,
            child: Text.rich(
              digit.toString().toTextSpan(context),
              textAlign: TextAlign.center,
              style: TextStyle(fontSize: context.textTheme.displayMedium?.fontSize),
            ),
          ),
        ),
      ),
    );
  }
}
