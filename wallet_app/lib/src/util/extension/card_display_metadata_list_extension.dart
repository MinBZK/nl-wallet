import '../../domain/model/card/metadata/card_display_metadata.dart';
import '../../domain/model/localized_text.dart';
import 'object_extension.dart';

extension CardDisplayMetadataExtension on List<CardDisplayMetadata> {
  LocalizedText? get name {
    return asMap().map((index, entry) => MapEntry(entry.language, entry.name)).takeIf((it) => it.isNotEmpty);
  }

  LocalizedText? get description {
    final dataWithDescription = where((entry) => entry.description != null);
    return dataWithDescription
        .toList()
        .asMap()
        .map((index, entry) => MapEntry(entry.language, entry.description!))
        .takeIf((it) => it.isNotEmpty);
  }

  LocalizedText? get rawSummary {
    final dataWithSummary = where((entry) => entry.rawSummary != null);
    return dataWithSummary
        .toList()
        .asMap()
        .map((index, entry) => MapEntry(entry.language, entry.rawSummary!))
        .takeIf((it) => it.isNotEmpty);
  }
}
