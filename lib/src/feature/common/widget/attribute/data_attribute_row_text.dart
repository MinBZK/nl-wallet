import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/data_attribute.dart';

class DataAttributeRowText extends StatelessWidget {
  final DataAttribute attribute;

  const DataAttributeRowText({required this.attribute, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          attribute.label,
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
