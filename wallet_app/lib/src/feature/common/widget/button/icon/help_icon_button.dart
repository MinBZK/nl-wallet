import 'package:flutter/material.dart';

import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/string_extension.dart';

class HelpIconButton extends StatelessWidget {
  final VoidCallback onPressed;

  const HelpIconButton({required this.onPressed, super.key});

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      attributedLabel: context.l10n.generalWCAGHelp.toAttributedString(context),
      onTap: onPressed,
      excludeSemantics: true,
      child: IconButton(
        onPressed: onPressed,
        icon: const Icon(Icons.help_outline_rounded),
        tooltip: context.l10n.generalWCAGHelp,
      ),
    );
  }
}
