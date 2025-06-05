import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';

class CompactListItem extends StatelessWidget {
  /// The main text displayed in the list item.
  final Widget label;

  /// The secondary text displayed below the label.
  final Widget subtitle;

  /// The optional leading icon displayed before the text.
  final Widget? icon;

  const CompactListItem({super.key, this.icon, required this.label, required this.subtitle});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          icon ?? const SizedBox.shrink(),
          SizedBox(width: icon == null ? 0 : 16),
          Expanded(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                DefaultTextStyle(style: context.textTheme.bodyMedium!, child: label),
                DefaultTextStyle(style: context.textTheme.titleMedium!, child: subtitle),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
