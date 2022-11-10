import 'package:flutter/material.dart';

import '../../../domain/model/data_attribute.dart';

class MissingDataAttributeRow extends StatelessWidget {
  final DataAttribute attribute;

  const MissingDataAttributeRow({required this.attribute, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        const Icon(Icons.do_not_disturb_on_outlined, size: 20),
        const SizedBox(width: 16),
        Text(
          attribute.type,
          style: Theme.of(context).textTheme.bodyText1,
        ),
      ],
    );
  }
}
