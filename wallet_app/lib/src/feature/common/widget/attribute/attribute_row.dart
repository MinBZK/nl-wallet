import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import 'data_attribute_row.dart';
import 'missing_attribute_row.dart';

/// Renders an [Attribute] to the screen
class AttributeRow extends StatelessWidget {
  final Attribute attribute;

  const AttributeRow({required this.attribute, super.key});

  @override
  Widget build(BuildContext context) {
    final attribute = this.attribute;
    switch (attribute) {
      case DataAttribute():
        return DataAttributeRow(attribute: attribute);
      case MissingAttribute():
        final label = attribute.label.l10nValue(context);
        return MissingAttributeRow(label: label);
    }
  }
}
