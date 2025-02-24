import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';

class KeyboardBackspaceKey extends StatelessWidget {
  final VoidCallback? onBackspacePressed;
  final VoidCallback? onBackspaceLongPressed;

  const KeyboardBackspaceKey({
    this.onBackspacePressed,
    this.onBackspaceLongPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    final onPressed = onBackspacePressed == null ? null : () => onBackspacePressed!();
    final onLongPress = onBackspaceLongPressed == null ? null : () => onBackspaceLongPressed!();
    return Expanded(
      child: MergeSemantics(
        child: Semantics(
          keyboardKey: true,
          button: true,
          onTap: onPressed,
          onLongPress: onLongPress,
          onLongPressHint: context.l10n.pinKeyboardWCAGBackspaceLongPressHint,
          attributedLabel: context.l10n.pinKeyboardWCAGBackspaceLabel.toAttributedString(context),
          child: TextButton.icon(
            onLongPress: onLongPress,
            onPressed: onPressed,
            label: const SizedBox.shrink(),
            style: context.theme.iconButtonTheme.style?.copyWith(
              shape: WidgetStateProperty.all(
                const RoundedRectangleBorder(borderRadius: BorderRadius.zero),
              ),
            ),
            icon: const Icon(Icons.keyboard_backspace_rounded),
          ),
        ),
      ),
    );
  }
}
