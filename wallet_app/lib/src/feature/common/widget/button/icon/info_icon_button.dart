import 'package:flutter/material.dart';

import '../../../../../navigation/wallet_routes.dart';
import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/string_extension.dart';

class InfoIconButton extends StatelessWidget {
  final VoidCallback? onPressed;

  const InfoIconButton({this.onPressed, super.key});

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      attributedLabel: context.l10n.generalWCAGInfo.toAttributedString(context),
      onTap: onPressed ?? () => Navigator.pushNamed(context, WalletRoutes.aboutRoute),
      excludeSemantics: true,
      child: IconButton(
        onPressed: onPressed ?? () => Navigator.pushNamed(context, WalletRoutes.aboutRoute),
        icon: const Icon(Icons.info_outline_rounded),
        tooltip: context.l10n.generalWCAGInfo,
      ),
    );
  }
}
