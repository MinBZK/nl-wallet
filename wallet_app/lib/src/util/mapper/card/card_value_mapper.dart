import '../../../domain/model/attribute/data_value.dart';
import '../../../wallet_core/wallet_core.dart';

class CardValueMapper {
  CardValueMapper();

  DataValue map(CardValue input) {
    switch (input) {
      case final CardValue_String string:
        return DataValueString(string.value);
      case final CardValue_Integer integer:
        return DataValueInteger(integer.value);
      case final CardValue_Double double:
        return DataValueDouble(double.value);
      case final CardValue_Boolean boolean:
        return DataValueBoolean(boolean.value);
      case final CardValue_Date date:
        return DataValueDate(date.value);
      default:
        return DataValueString(input.value.toString());
    }
  }
}
