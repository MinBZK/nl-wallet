import 'package:wallet_core/core.dart';

import '../../../../domain/model/localized_text.dart';
import '../../mapper.dart';

class ClaimDisplayMetadataMapper extends Mapper<List<ClaimDisplayMetadata>, LocalizedText> {
  ClaimDisplayMetadataMapper();

  @override
  LocalizedText map(List<ClaimDisplayMetadata> input) =>
      input.asMap().map((key, value) => MapEntry(value.lang, value.label));
}
