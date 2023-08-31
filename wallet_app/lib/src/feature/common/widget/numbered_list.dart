import 'package:collection/collection.dart';
import 'package:flutter/material.dart';

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
      children: items.mapIndexed((index, point) => _buildTableRow(index, point)).toList(),
    );
  }

  TableRow _buildTableRow(int index, String point) {
    return TableRow(
      children: [
        Text('  ${index + 1}.  '),
        Text(point),
      ],
    );
  }
}
