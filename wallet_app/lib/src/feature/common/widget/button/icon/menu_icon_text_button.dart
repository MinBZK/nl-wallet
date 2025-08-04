import 'package:flutter/material.dart';

import '../../../../../navigation/wallet_routes.dart';
import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/string_extension.dart';

class MenuIconTextButton extends StatelessWidget {
  final VoidCallback? onPressed;

  const MenuIconTextButton({this.onPressed, super.key});

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      tooltip: context.l10n.generalWCAGMenu,
      attributedLabel: context.l10n.dashboardScreenMenuWCAGLabel.toAttributedString(context),
      excludeSemantics: true,
      onTap: onPressed ?? () => Navigator.pushNamed(context, WalletRoutes.menuRoute),
      child: TextButton.icon(
        onPressed: onPressed ?? () => Navigator.pushNamed(context, WalletRoutes.menuRoute),
        icon: const Icon(Icons.menu_rounded),
        label: Text.rich(context.l10n.dashboardScreenTitle.toTextSpan(context)),
        style: context.theme.iconButtonTheme.style
            ?.copyWith(padding: const WidgetStatePropertyAll(EdgeInsets.symmetric(horizontal: 16))),
      ),
    );
  }
}
