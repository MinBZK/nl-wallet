import 'package:flutter/material.dart';

import '../../../../theme/base_wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';

class VerticalListItem extends StatelessWidget {
  /// The main text displayed in the vertical list item.
  final Widget title;

  /// The secondary text displayed below the label.
  final Widget subtitle;

  /// The optional leading icon displayed to the left of the text.
  final Widget? icon;

  /// The optional button displayed at the bottom of the list item.
  final Widget? button;

  const VerticalListItem({super.key, this.icon, required this.title, required this.subtitle, this.button});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Align(alignment: Alignment.centerLeft, child: icon ?? const SizedBox.shrink()),
          SizedBox(height: icon == null ? 0 : 16),
          Semantics(
            header: true,
            child: DefaultTextStyle(
              style: BaseWalletTheme.headlineExtraSmallTextStyle.copyWith(
                color: context.textTheme.titleMedium?.color,
              ),
              child: title,
            ),
          ),
          const SizedBox(height: 8),
          DefaultTextStyle(style: context.textTheme.bodyLarge!, child: subtitle),
          SizedBox(height: button == null ? 0 : 12),
          if (button != null) button!,
        ],
      ),
    );
  }
}
