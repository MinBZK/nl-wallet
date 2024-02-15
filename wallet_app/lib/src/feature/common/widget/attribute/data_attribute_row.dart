import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/formatter/attribute_value_formatter.dart';

class DataAttributeRow extends StatelessWidget {
  final DataAttribute attribute;

  const DataAttributeRow({required this.attribute, super.key});

  @override
  Widget build(BuildContext context) {
    return MergeSemantics(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            attribute.label.l10nValue(context),
            style: context.textTheme.bodySmall,
          ),
          Text(
            attribute.value.prettyPrint(context),
            style: context.textTheme.titleMedium,
          ),
        ],
      ),
    );
  }
}
