import 'package:flutter/material.dart';

import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/string_extension.dart';

/// Similar to the normal [BackButton] widget, but always uses the same icon (ios/android).
class BackIconButton extends StatelessWidget {
  final VoidCallback? onPressed;

  const BackIconButton({this.onPressed, super.key});

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      onTap: onPressed ?? () => Navigator.pop(context),
      attributedLabel: context.l10n.generalWCAGBack.toAttributedString(context),
      excludeSemantics: true,
      child: Center(
        child: IconButton(
          onPressed: onPressed ?? () => Navigator.pop(context),
          icon: const Icon(Icons.arrow_back_rounded),
          tooltip: context.l10n.generalWCAGBack,
        ),
      ),
    );
  }
}
