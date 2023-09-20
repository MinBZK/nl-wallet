import '../../../wallet_core/wallet_core.dart';

class CardAttributeLabelMapper {
  String map(List<LocalizedString> input, String languageCode) {
    return input
        .firstWhere(
          (element) => element.language == languageCode,
          orElse: () => input.first,
        )
        .value;
  }
}
