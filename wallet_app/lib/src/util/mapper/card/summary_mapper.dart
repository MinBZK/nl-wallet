import 'package:collection/collection.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../extension/card_display_metadata_list_extension.dart';
import '../../extension/string_extension.dart';
import '../../formatter/attribute_value_formatter.dart';
import '../mapper.dart';

class CardSummaryMapper extends Mapper<WalletCard, LocalizedText> {
  @override
  LocalizedText map(WalletCard input) {
    final rawSummary = input.metadata.rawSummary;
    if (rawSummary == null) return ''.untranslated;
    return rawSummary.map((locale, template) {
      // Replace all placeholders
      final String result = template.replaceAllMapped(
        RegExp(r'{{([a-zA-Z_][\w]*)}}'),
        (match) {
          // Locate the svgId (key of an attribute)
          final svgId = match.group(1);
          if (svgId == null) return '';
          // Find the corresponding DataAttribute
          final attribute = input.attributes.firstWhereOrNull((attribute) => attribute.svgId == svgId);
          if (attribute == null) return '';
          // Return the localized value
          return AttributeValueFormatter.formatWithLocale(locale, attribute.value);
        },
      );

      return MapEntry(locale, result);
    });
  }
}
