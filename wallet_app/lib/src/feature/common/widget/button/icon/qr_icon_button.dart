import 'package:flutter/material.dart';

import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/string_extension.dart';
import '../../../sheet/qr_action_sheet.dart';

class QrIconButton extends StatelessWidget {
  final VoidCallback? onPressed;

  const QrIconButton({this.onPressed, super.key});

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      attributedLabel: context.l10n.generalWCAGQr.toAttributedString(context),
      onTap: onPressed ?? () => QrActionSheet.show(context),
      excludeSemantics: true,
      child: IconButton(
        onPressed: onPressed ?? () => QrActionSheet.show(context),
        icon: const Icon(Icons.qr_code_rounded),
        tooltip: context.l10n.generalWCAGQr,
      ),
    );
  }
}
