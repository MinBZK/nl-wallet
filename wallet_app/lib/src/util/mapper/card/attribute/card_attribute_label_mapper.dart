import '../../../../wallet_core/wallet_core.dart';
import '../../locale_mapper.dart';

const _kFallbackLocalizedString = LocalizedString(language: '', value: '');

class CardAttributeLabelMapper extends LocaleMapper<List<LocalizedString>, String> {
  @override
  String map(Locale locale, List<LocalizedString> input) {
    return input
        .firstWhere(
          (element) => element.language == locale.languageCode,
          orElse: () => input.isNotEmpty ? input.first : _kFallbackLocalizedString,
        )
        .value;
  }
}
