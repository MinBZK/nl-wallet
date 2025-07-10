import 'package:flutter/material.dart';

import '../../../theme/base_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../util/extension/text_style_extension.dart';

class KeyboardDigitKey extends StatelessWidget {
  final int digit;
  final Function(int)? onKeyPressed;

  const KeyboardDigitKey({required this.digit, this.onKeyPressed, super.key});

  @override
  Widget build(BuildContext context) {
    final onPressed = (onKeyPressed == null ? null : () => onKeyPressed!(digit));
    return Expanded(
      child: MergeSemantics(
        child: Semantics(
          keyboardKey: true,
          button: true,
          onTap: onPressed,
          child: TextButton(
            onPressed: onPressed,
            style: context.theme.textButtonTheme.style?.copyWith(
              shape: WidgetStateProperty.all(
                const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
              ),
              textStyle: WidgetStateTextStyle.resolveWith((states) {
                final textStyle = context.textTheme.headlineMedium!;
                return states.isPressedOrFocused ? textStyle.underlined : textStyle;
              }),
            ),
            child: OverflowBox(
              maxWidth: double.infinity,
              maxHeight: double.infinity,
              alignment: Alignment.center,
              child: Text.rich(
                digit.toString().toTextSpan(context),
                textAlign: TextAlign.center,
              ),
            ),
          ),
        ),
      ),
    );
  }
}
