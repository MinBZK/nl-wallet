import 'package:flutter/material.dart';

import '../model/data_attribute.dart';

class DataAttributeRow extends StatelessWidget {
  final DataAttribute attribute;

  const DataAttributeRow({required this.attribute, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          attribute.type,
          style: Theme.of(context).textTheme.caption,
        ),
        Text(
          attribute.value,
          style: Theme.of(context).textTheme.subtitle1,
        ),
      ],
    );
  }
}
