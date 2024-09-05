import 'package:flutter/material.dart';

import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/string_extension.dart';
import '../../../screen/placeholder_screen.dart';

class HelpIconButton extends StatelessWidget {
  final VoidCallback? onPressed;

  const HelpIconButton({this.onPressed, super.key});

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      attributedLabel: context.l10n.generalWCAGHelp.toAttributedString(context),
      onTap: onPressed ?? () => PlaceholderScreen.showHelp(context, secured: false),
      excludeSemantics: true,
      child: IconButton(
        onPressed: onPressed ?? () => PlaceholderScreen.showHelp(context, secured: false),
        icon: const Icon(Icons.help_outline_rounded),
        tooltip: context.l10n.generalWCAGHelp,
      ),
    );
  }
}
