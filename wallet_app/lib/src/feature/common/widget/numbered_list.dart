import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';

class NumberedList extends StatelessWidget {
  final List<String> items;

  const NumberedList({required this.items, super.key});

  @override
  Widget build(BuildContext context) {
    return ListView.builder(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemCount: items.length,
      itemBuilder: (c, i) {
        return Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('  ${i + 1}.  ', style: context.textTheme.bodyLarge),
            Expanded(child: Text(items[i], style: context.textTheme.bodyLarge)),
          ],
        );
      },
    );
  }
}
