import 'package:wallet_core/core.dart';

import '../../../../domain/model/localized_text.dart';
import '../../../extension/locale_extension.dart';
import '../../mapper.dart';

class ClaimDisplayMetadataMapper extends Mapper<List<ClaimDisplayMetadata>, LocalizedText> {
  ClaimDisplayMetadataMapper();

  @override
  LocalizedText map(List<ClaimDisplayMetadata> input) {
    return input.asMap().map((key, value) {
      return MapEntry(LocaleExtension.parseLocale(value.lang), value.label);
    });
  }
}
