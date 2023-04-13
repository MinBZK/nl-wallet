import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/attribute/requested_attribute.dart';
import '../../../../domain/model/attribute/ui_attribute.dart';
import 'data_attribute_row.dart';
import 'requested_attribute_row.dart';
import 'ui_attribute_row.dart';

class AttributeRow extends StatelessWidget {
  final Attribute attribute;

  const AttributeRow({required this.attribute, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    if (attribute is DataAttribute) {
      return DataAttributeRow(attribute: attribute as DataAttribute);
    }
    if (attribute is RequestedAttribute) {
      return RequestedAttributeRow(attribute: attribute as RequestedAttribute);
    }
    if (attribute is UiAttribute) {
      return UiAttributeRow(attribute: attribute as UiAttribute);
    }
    throw UnsupportedError('Unsupported Attribute type: ${attribute.runtimeType}');
  }
}
