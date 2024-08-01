import 'package:flutter/material.dart';

import '../../../../../util/extension/build_context_extension.dart';

const kCloseIconButtonKey = Key('close_icon_button');

class CloseIconButton extends StatelessWidget {
  final VoidCallback? onPressed;

  const CloseIconButton({this.onPressed, super.key});

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      label: context.l10n.generalWCAGClose,
      onTap: onPressed ?? () => Navigator.pop(context),
      excludeSemantics: true,
      child: IconButton(
        key: kCloseIconButtonKey,
        onPressed: onPressed ?? () => Navigator.pop(context),
        icon: const Icon(Icons.close_rounded),
        tooltip: context.l10n.generalWCAGClose,
      ),
    );
  }
}
