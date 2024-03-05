import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import 'icon_row.dart';

class BulletList extends StatelessWidget {
  final List<String> items;
  final IconData? icon;

  const BulletList({required this.items, this.icon, super.key});

  @override
  Widget build(BuildContext context) {
    if (items.isEmpty) return const SizedBox.shrink();
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: items.map((point) {
          return IconRow(
            padding: const EdgeInsets.symmetric(vertical: 4),
            icon: SizedBox(
              height: 24,
              width: 24,
              child: Icon(
                icon ?? Icons.check,
                color: context.colorScheme.primary,
                size: 18,
              ),
            ),
            text: Text(point),
          );
        }).toList(),
      ),
    );
  }
}
