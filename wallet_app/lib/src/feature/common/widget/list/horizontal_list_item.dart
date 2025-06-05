import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';

class HorizontalListItem extends StatelessWidget {
  /// The main text displayed in the horizontal list item.
  final Widget label;

  /// The secondary text displayed below the label.
  final Widget subtitle;

  /// The optional leading icon displayed before the text in a horizontal layout.
  final Widget? icon;

  const HorizontalListItem({super.key, this.icon, required this.label, required this.subtitle});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          icon ?? const SizedBox.shrink(),
          SizedBox(width: icon == null ? 0 : 16),
          Expanded(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                DefaultTextStyle(style: context.textTheme.titleMedium!, child: label),
                const SizedBox(height: 8),
                DefaultTextStyle(style: context.textTheme.bodyLarge!, child: subtitle),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
