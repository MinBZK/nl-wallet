import '../../../domain/model/attribute/data_value.dart';
import '../../../wallet_core/wallet_core.dart';

class CardValueMapper {
  CardValueMapper();

  DataValue map(CardValue input) {
    return input.map(
      string: (input) => DataValueString(input.value),
      boolean: (input) => DataValueBoolean(input.value),
      date: (input) => DataValueDate(input.value),
      gender: (input) => DataValueGender(input.value),
    );
  }
}
