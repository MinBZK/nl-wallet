import 'package:flutter/cupertino.dart';
import '../../../l10n/generated/app_localizations.dart';

import '../../domain/model/attribute/attribute.dart';
import '../extension/build_context_extension.dart';

class AttributeValueFormatter {
  static String format(BuildContext context, AttributeValue attributeValue) {
    return switch (attributeValue) {
      StringValue() => attributeValue.value,
      BooleanValue() => attributeValue.value ? context.l10n.cardValueTrue : context.l10n.cardValueFalse,
      NumberValue() => attributeValue.value.toString(),
    };
  }

  static String formatWithLocale(Locale locale, AttributeValue attributeValue) {
    final l10n = lookupAppLocalizations(locale);
    return switch (attributeValue) {
      StringValue() => attributeValue.value,
      BooleanValue() => attributeValue.value ? l10n.cardValueTrue : l10n.cardValueFalse,
      NumberValue() => attributeValue.value.toString(),
    };
  }
}

extension AttributeValueExtension on AttributeValue {
  String prettyPrint(BuildContext context) => AttributeValueFormatter.format(context, this);
}
