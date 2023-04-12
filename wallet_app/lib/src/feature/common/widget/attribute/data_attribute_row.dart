import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/data_attribute.dart';
import 'data_attribute_row_image.dart';
import 'data_attribute_row_text.dart';

class DataAttributeRow extends StatelessWidget {
  final DataAttribute attribute;

  const DataAttributeRow({required this.attribute, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    switch (attribute.valueType) {
      case AttributeValueType.text:
        return DataAttributeRowText(attribute: attribute);
      case AttributeValueType.image:
        return Align(
          alignment: Alignment.centerLeft,
          child: DataAttributeRowImage(
            image: AssetImage(attribute.value),
            label: attribute.label,
          ),
        );
    }
  }
}
