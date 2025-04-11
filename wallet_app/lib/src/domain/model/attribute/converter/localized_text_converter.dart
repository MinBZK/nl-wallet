import 'package:json_annotation/json_annotation.dart';

import '../../../../util/extension/locale_extension.dart';
import '../../localized_text.dart';

class LocalizedTextConverter extends JsonConverter<LocalizedText, Map<String, dynamic>> {
  const LocalizedTextConverter();

  @override
  LocalizedText fromJson(Map<String, dynamic> json) {
    return json.map(
      (key, value) => MapEntry(
        LocaleExtension.parseLocale(key),
        value,
      ),
    );
  }

  @override
  Map<String, dynamic> toJson(LocalizedText object) {
    return object.map(
      (key, value) => MapEntry(
        key.toLanguageTag(),
        value,
      ),
    );
  }
}
