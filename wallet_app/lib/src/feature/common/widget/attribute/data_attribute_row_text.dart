import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../util/extension/build_context_extension.dart';

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
          style: context.textTheme.bodySmall,
        ),
        Text(
          attribute.value,
          style: context.textTheme.titleMedium,
        ),
      ],
    );
  }
}
