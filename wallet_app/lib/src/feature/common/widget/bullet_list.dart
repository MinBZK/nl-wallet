import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import 'bullet_list_dot.dart';
import 'icon_row.dart';
import 'text/body_text.dart';

class BulletList extends StatelessWidget {
  final List<String> items;
  final Widget icon;
  final CrossAxisAlignment? rowCrossAxisAlignment;
  final EdgeInsets? rowPadding;

  const BulletList({
    required this.items,
    this.icon = const BulletListDot(),
    this.rowCrossAxisAlignment = .start,
    this.rowPadding,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    if (items.isEmpty) return const SizedBox.shrink();
    final scale = _calculateTextHeight(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: items.map((point) {
        return IconRow(
          crossAxisAlignment: rowCrossAxisAlignment,
          padding: rowPadding ?? EdgeInsets.zero,
          icon: SizedBox(
            height: scale,
            width: 24,
            child: icon,
          ),
          text: BodyText(point),
        );
      }).toList(),
    );
  }

  /// Calculates the scaled line height for [BodyText] based on the current theme.
  double _calculateTextHeight(BuildContext context) {
    final textStyle = context.textTheme.bodyLarge;
    final fontSize = textStyle?.fontSize ?? 24.0;
    final heightScale = textStyle?.height ?? 1.0;
    return context.textScaler.scale(fontSize * heightScale);
  }
}
