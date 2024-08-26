import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';

class KeyboardBackspaceKey extends StatelessWidget {
  final VoidCallback? onBackspacePressed;
  final VoidCallback? onBackspaceLongPressed;
  final Color? color;

  const KeyboardBackspaceKey({
    this.onBackspacePressed,
    this.onBackspaceLongPressed,
    this.color,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: Semantics(
        button: true,
        keyboardKey: true,
        onLongPressHint: context.l10n.pinKeyboardWCAGBackspaceLongPressHint,
        attributedLabel: context.l10n.pinKeyboardWCAGBackspaceLabel.toAttributedString(context),
        child: InkWell(
          onLongPress: onBackspaceLongPressed == null ? null : () => onBackspaceLongPressed!(),
          onTap: onBackspacePressed == null ? null : () => onBackspacePressed!(),
          child: Icon(
            Icons.keyboard_backspace_rounded,
            color: color,
            size: 24,
          ),
        ),
      ),
    );
  }
}
