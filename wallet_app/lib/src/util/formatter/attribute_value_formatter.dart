import 'package:flutter/cupertino.dart';
import 'package:intl/intl.dart';

import '../../../l10n/generated/app_localizations.dart';
import '../../domain/model/attribute/attribute.dart';
import '../extension/build_context_extension.dart';
import '../extension/locale_extension.dart';

class AttributeValueFormatter {
  static String format(BuildContext context, AttributeValue attributeValue, {bool inline = false}) =>
      formatWithLocale(context.activeLocale, attributeValue, inline: inline);

  /// Set [inline] for single line contexts, such as the summary on a card front. Composite values are
  /// then joined with a comma instead of being spread over multiple lines.
  static String formatWithLocale(Locale locale, AttributeValue attribute, {bool inline = false}) {
    final l10n = locale.l10n;
    return switch (attribute) {
      StringValue() => _formatStringValue(locale, attribute),
      BooleanValue() => _formatBooleanValue(locale, attribute),
      NumberValue() => '${attribute.value}',
      DateValue() => _prettyPrintDateTime(locale, attribute.value),
      ArrayValue() => _formatArrayValue(locale, attribute, inline),
      BytesValue() => l10n.cardValueUnsupported,
      ImageValue() => l10n.cardValueImage,
      MapValue() => _formatMapValue(locale, attribute, inline),
      NullValue() => l10n.cardValueNull,
    };
  }

  static String _formatStringValue(Locale locale, StringValue attribute) =>
      attribute.value.isEmpty ? locale.l10n.cardValueEmpty : attribute.value;

  static String _formatBooleanValue(Locale locale, BooleanValue attribute) =>
      attribute.value ? locale.l10n.cardValueTrue : locale.l10n.cardValueFalse;

  static String _formatArrayValue(Locale locale, ArrayValue attribute, bool inline) {
    if (attribute.value.isEmpty) return locale.l10n.cardValueEmptyList;
    final entries = attribute.value.map((it) => formatWithLocale(locale, it, inline: inline));
    return inline ? entries.join(', ') : entries.map((it) => '  • $it').join('\n');
  }

  /// Translates the keys of a [MapValue], such as the entries of an mDL driving privilege or of an
  /// mVRC vehicle owner. Returns null for a key that only groups the entries below it, such as the
  /// 'full_address' of an mVRC owner; those entries are rendered without a label of their own.
  // TODO(Daan): the core does not supply display metadata for nested claim paths yet, so these keys
  // arrive untranslated. Note that this matches on the bare key, so any attestation holding a
  // 'country' entry gets this label. Drop it once the attestation metadata declares display labels
  // for the entries of a composite value (PVW-6241).
  static String? formatMapKey(Locale locale, String key) {
    final label = _mapKeyLabels[key];
    return label == null ? key : label(locale.l10n);
  }

  static final Map<String, String? Function(AppLocalizations)> _mapKeyLabels = {
    'issue_date': (l10n) => l10n.cardValueMapKeyIssueDate,
    'expiry_date': (l10n) => l10n.cardValueMapKeyExpiryDate,
    'vehicle_category_code': (l10n) => l10n.cardValueMapKeyVehicleCategoryCode,
    'family_name': (l10n) => l10n.cardValueMapKeyFamilyName,
    'given_name': (l10n) => l10n.cardValueMapKeyGivenName,
    'full_address': (_) => null,
    'address': (l10n) => l10n.cardValueMapKeyAddress,
    'city': (l10n) => l10n.cardValueMapKeyCity,
    'postal_code': (l10n) => l10n.cardValueMapKeyPostalCode,
    'country': (l10n) => l10n.cardValueMapKeyCountry,
  };

  static String _formatMapValue(Locale locale, MapValue attribute, bool inline) {
    if (attribute.value.isEmpty) return locale.l10n.cardValueEmptyList;
    final entries = attribute.value.entries.map((it) {
      final key = formatMapKey(locale, it.key);
      final value = formatWithLocale(locale, it.value, inline: inline);
      return key == null ? value : '$key: $value';
    });
    return entries.join(inline ? ', ' : '\n');
  }

  static String _prettyPrintDateTime(Locale locale, DateTime dateTime) {
    return DateFormat.yMd(locale.toLanguageTag()).format(dateTime);
  }
}

extension AttributeValueExtension on AttributeValue {
  String prettyPrint(BuildContext context, {bool inline = false}) =>
      AttributeValueFormatter.format(context, this, inline: inline);
}
