import 'package:wallet_core/core.dart';

import '../../../../domain/model/localized_text.dart';
import '../../../extension/locale_extension.dart';
import '../../mapper.dart';

class LocalizedLabelsMapper extends Mapper<List<LocalizedString>, LocalizedText> {
  LocalizedLabelsMapper();

  @override
  LocalizedText map(List<LocalizedString> input) {
    return input.asMap().map((key, value) {
      return MapEntry(LocaleExtension.parseLocale(value.language), value.value);
    });
  }
}
