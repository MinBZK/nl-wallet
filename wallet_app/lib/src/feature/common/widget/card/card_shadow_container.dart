import 'package:flutter/material.dart';

import '../../../../theme/wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';

/// A container that applies a card-style shadow behind its [child].
class CardShadowContainer extends StatelessWidget {
  /// The widget rendered inside the shadow container.
  final Widget child;

  const CardShadowContainer({required this.child, super.key});

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(
        color: context.colorScheme.surface,
        borderRadius: WalletTheme.kBorderRadius12,
        boxShadow: const [
          BoxShadow(
            color: Color(0x0000000D),
            blurRadius: 15,
            offset: Offset(0, 1),
          ),
          BoxShadow(
            color: Color(0x152A621A),
            blurRadius: 4,
            offset: Offset(0, 4),
          ),
        ],
      ),
      child: child,
    );
  }
}
