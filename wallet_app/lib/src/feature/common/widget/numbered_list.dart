import 'package:collection/collection.dart';
import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

class NumberedList extends StatelessWidget {
  final List<String> items;

  const NumberedList({required this.items, super.key});

  @override
  Widget build(BuildContext context) {
    if (items.isEmpty) return const SizedBox.shrink();
    return Table(
      columnWidths: const {
        0: IntrinsicColumnWidth(),
      },
      children: items.mapIndexed((index, point) => _buildTableRow(index, point, context.textTheme.bodyLarge)).toList(),
    );
  }

  TableRow _buildTableRow(int index, String point, TextStyle? textStyle) {
    return TableRow(
      children: [
        Text('  ${index + 1}.  ', style: textStyle),
        Text(point, style: textStyle),
      ],
    );
  }
}
