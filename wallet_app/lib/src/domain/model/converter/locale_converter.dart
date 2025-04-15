import 'dart:ui';

import 'package:json_annotation/json_annotation.dart';

import '../../../util/extension/locale_extension.dart';

class LocaleConverter extends JsonConverter<Locale, String> {
  const LocaleConverter();

  @override
  Locale fromJson(String json) => LocaleExtension.parseLocale(json);

  @override
  String toJson(Locale object) => object.toLanguageTag();
}
