import '../../../../../bridge_generated.dart';
import '../../../../domain/model/attribute/attribute.dart';
import '../../mapper.dart';

class LocalizedLabelsMapper extends Mapper<List<LocalizedString>, LocalizedText> {
  LocalizedLabelsMapper();

  @override
  LocalizedText map(List<LocalizedString> input) =>
      input.asMap().map((key, value) => MapEntry(value.language, value.value));
}
