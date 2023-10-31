import 'package:json_annotation/json_annotation.dart';

import '../../../../wallet_core/wallet_core.dart';

class LocalizedStringConverter extends JsonConverter<LocalizedString, Map<String, dynamic>> {
  const LocalizedStringConverter();

  @override
  LocalizedString fromJson(Map<String, dynamic> json) {
    final result = json.entries.first;
    return LocalizedString(language: result.key, value: result.value);
  }

  @override
  Map<String, dynamic> toJson(LocalizedString object) {
    return {object.language: object.value};
  }
}
