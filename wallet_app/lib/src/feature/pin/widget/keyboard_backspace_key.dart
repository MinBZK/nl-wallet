import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

class KeyboardBackspaceKey extends StatelessWidget {
  final VoidCallback? onBackspacePressed;

  const KeyboardBackspaceKey({this.onBackspacePressed, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: Semantics(
        button: true,
        label: context.l10n.pinKeyboardWCAGBackspaceLabel,
        child: InkWell(
          onTap: onBackspacePressed == null ? null : () => onBackspacePressed!(),
          child: const Icon(Icons.backspace),
        ),
      ),
    );
  }
}
