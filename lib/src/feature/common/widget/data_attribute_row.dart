import 'package:flutter/material.dart';

import '../../../domain/model/data_attribute.dart';
import 'data_attribute_row_image.dart';
import 'data_attribute_row_missing.dart';
import 'data_attribute_row_text.dart';

class DataAttributeRow extends StatelessWidget {
  final DataAttribute attribute;

  const DataAttributeRow({required this.attribute, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    switch (attribute.valueType) {
      case DataAttributeValueType.text:
        if (attribute.value?.isNotEmpty ?? false) {
          return DataAttributeRowText(attribute: attribute);
        } else {
          return DataAttributeRowMissing(attribute: attribute);
        }
      case DataAttributeValueType.image:
        return Align(
          alignment: Alignment.centerLeft,
          child: DataAttributeRowImage(image: AssetImage(attribute.value!), label: attribute.label),
        );
    }
  }
}
