import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/attribute/requested_attribute.dart';
import 'data_attribute_row_image.dart';
import 'data_attribute_row_missing.dart';

class RequestedAttributeRow extends StatelessWidget {
  final RequestedAttribute attribute;

  const RequestedAttributeRow({required this.attribute, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    switch (attribute.valueType) {
      case AttributeValueType.text:
        return DataAttributeRowMissing(label: attribute.name);
      case AttributeValueType.image:
        return Align(
          alignment: Alignment.centerLeft,
          child: DataAttributeRowImage(
            image: const AssetImage('assets/non-free/images/image_attribute_placeholder.png'),
            label: attribute.name,
          ),
        );
    }
  }
}
