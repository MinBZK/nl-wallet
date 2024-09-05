import 'package:flutter/material.dart';

import '../../../../../navigation/wallet_routes.dart';
import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/string_extension.dart';

class MenuIconButton extends StatelessWidget {
  final VoidCallback? onPressed;

  const MenuIconButton({this.onPressed, super.key});

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      attributedLabel: context.l10n.generalWCAGMenu.toAttributedString(context),
      excludeSemantics: true,
      onTap: onPressed ?? () => Navigator.pushNamed(context, WalletRoutes.menuRoute),
      child: IconButton(
        tooltip: context.l10n.generalWCAGMenu,
        onPressed: onPressed ?? () => Navigator.pushNamed(context, WalletRoutes.menuRoute),
        icon: const Icon(Icons.menu_rounded),
      ),
    );
  }
}
