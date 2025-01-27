import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import 'data_attribute_row.dart';
import 'missing_attribute_row.dart';
import 'ui_attribute_row.dart';

/// Renders an [Attribute] to the screen
class AttributeRow extends StatelessWidget {
  final Attribute attribute;

  const AttributeRow({required this.attribute, super.key});

  @override
  Widget build(BuildContext context) {
    switch (attribute) {
      case DataAttribute():
        return DataAttributeRow(attribute: attribute as DataAttribute);
      case UiAttribute():
        return UiAttributeRow(attribute: attribute as UiAttribute);
      case MissingAttribute():
        final label = (attribute as MissingAttribute).label.l10nValue(context);
        return MissingAttributeRow(label: label);
    }
  }
}
