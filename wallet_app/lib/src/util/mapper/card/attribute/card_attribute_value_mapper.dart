import 'package:wallet_core/core.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../mapper.dart';

class CardAttributeValueMapper extends Mapper<AttestationValue, AttributeValue> {
  CardAttributeValueMapper();

  @override
  AttributeValue map(AttestationValue input) {
    return input.map(
      string: (input) => StringValue(input.value),
      boolean: (input) => BooleanValue(input.value),
      number: (input) => NumberValue(input.value),
    );
  }
}
