import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/attribute/attribute.dart';
import '../../mapper.dart';

class CardAttributeValueMapper extends Mapper<core.AttributeValue, AttributeValue> {
  CardAttributeValueMapper();

  @override
  AttributeValue map(core.AttributeValue input) {
    return input.map(
      string: (input) => StringValue(input.value),
      boolean: (input) => BooleanValue(input.value),
      number: (input) => NumberValue(input.value),
      date: (input) => DateValue(DateTime.parse(input.value)),
    );
  }
}
