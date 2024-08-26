import 'package:flutter/material.dart';

import '../../../../../navigation/wallet_routes.dart';
import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/string_extension.dart';

class QrIconButton extends StatelessWidget {
  final VoidCallback? onPressed;

  const QrIconButton({this.onPressed, super.key});

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      attributedLabel: context.l10n.generalWCAGQr.toAttributedString(context),
      onTap: onPressed ?? () => Navigator.pushNamed(context, WalletRoutes.qrRoute),
      excludeSemantics: true,
      child: IconButton(
        onPressed: onPressed ?? () => Navigator.pushNamed(context, WalletRoutes.qrRoute),
        icon: const Icon(Icons.qr_code_rounded),
        tooltip: context.l10n.generalWCAGQr,
      ),
    );
  }
}
