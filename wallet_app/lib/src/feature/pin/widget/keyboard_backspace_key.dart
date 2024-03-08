import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

class KeyboardBackspaceKey extends StatelessWidget {
  final VoidCallback? onBackspacePressed;
  final Color? color;

  const KeyboardBackspaceKey({
    this.onBackspacePressed,
    this.color,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: Semantics(
        button: true,
        label: context.l10n.pinKeyboardWCAGBackspaceLabel,
        child: InkWell(
          onTap: onBackspacePressed == null ? null : () => onBackspacePressed!(),
          child: Icon(
            Icons.keyboard_backspace_rounded,
            color: color,
          ),
        ),
      ),
    );
  }
}
