import 'package:wallet_core/core.dart';

extension StringExtension on String {
  List<LocalizedString> get untranslated => [LocalizedString(language: '', value: this)];

  List<LocalizedString> get dutch => [LocalizedString(language: 'nl', value: this)];
}
