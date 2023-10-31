import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/attribute/value/gender.dart';
import '../../../../wallet_core/wallet_core.dart';
import '../../mapper.dart';

class CardAttributeValueMapper extends Mapper<CardValue, AttributeValue> {
  CardAttributeValueMapper();

  @override
  AttributeValue map(CardValue input) {
    return input.map(
      string: (input) => StringValue(input.value),
      boolean: (input) => BooleanValue(input.value),
      date: (input) => DateValue(DateTime.parse(input.value)),
      gender: (input) {
        switch (input.value) {
          case GenderCardValue.Unknown:
            return const GenderValue(Gender.unknown);
          case GenderCardValue.Male:
            return const GenderValue(Gender.male);
          case GenderCardValue.Female:
            return const GenderValue(Gender.female);
          case GenderCardValue.NotApplicable:
            return const GenderValue(Gender.notApplicable);
        }
      },
    );
  }
}
