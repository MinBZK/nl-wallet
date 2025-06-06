import 'package:flutter/material.dart';

import 'icon_row.dart';
import 'text/body_text.dart';

class BulletList extends StatelessWidget {
  final List<String> items;
  final Widget icon;
  final CrossAxisAlignment? rowCrossAxisAlignment;
  final EdgeInsets? rowPadding;

  const BulletList({
    required this.items,
    required this.icon,
    this.rowCrossAxisAlignment,
    this.rowPadding,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    if (items.isEmpty) return const SizedBox.shrink();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: items.map((point) {
        return IconRow(
          crossAxisAlignment: rowCrossAxisAlignment,
          padding: rowPadding ?? EdgeInsets.zero,
          icon: SizedBox(
            height: 24,
            width: 24,
            child: icon,
          ),
          text: BodyText(point),
        );
      }).toList(),
    );
  }
}
