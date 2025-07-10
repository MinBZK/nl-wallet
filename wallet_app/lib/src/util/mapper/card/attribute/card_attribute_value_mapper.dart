import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/attribute/attribute.dart';
import '../../mapper.dart';

class CardAttributeValueMapper extends Mapper<core.AttributeValue, AttributeValue> {
  CardAttributeValueMapper();

  @override
  AttributeValue map(core.AttributeValue input) {
    return switch (input) {
      core.AttributeValue_String(:final value) => StringValue(value),
      core.AttributeValue_Boolean(:final value) => BooleanValue(value),
      core.AttributeValue_Number(:final value) => NumberValue(value),
      core.AttributeValue_Date(:final value) => DateValue(DateTime.parse(value)),
      core.AttributeValue_Null() => NullValue(),
    };
  }
}
