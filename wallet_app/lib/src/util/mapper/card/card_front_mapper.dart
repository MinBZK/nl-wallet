import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:wallet_core/core.dart';
import 'package:wallet_mock/mock.dart';

import '../../../domain/model/card_front.dart';
import '../../../domain/model/localized_text.dart';
import '../../../wallet_assets.dart';
import '../../extension/string_extension.dart';
import '../mapper.dart';

class CardFrontMapper extends Mapper<Attestation, CardFront> {
  final Mapper<Attestation, LocalizedText?> _subtitleMapper;

  CardFrontMapper(this._subtitleMapper);

  @override
  CardFront map(Attestation input) {
    final l10ns = AppLocalizations.supportedLocales.map(lookupAppLocalizations).toList();
    switch (input.attestationType) {
      case kPidDocType:
        return _getPidCardFront(l10ns, input);
      case kAddressDocType:
        return _getAddressCardFront(l10ns, input);
      case 'DIPLOMA_1':
        return _kMockDiplomaCardFront;
      case 'DIPLOMA_2':
        return _kMockMasterDiplomaCardFront;
      case kDrivingLicenseDocType:
        if (isRenewedLicense(input)) return _kMockDrivingLicenseRenewedCardFront;
        return _kMockDrivingLicenseCardFront;
      case 'HEALTH_INSURANCE':
        return _kMockHealthInsuranceCardFront;
      case 'VOG':
        return _kMockVOGCardFront;
    }
    throw Exception('Unknown docType: ${input.attestationType}');
  }

  CardFront _getPidCardFront(List<AppLocalizations> l10ns, Attestation input) {
    return CardFront(
      title: l10ns.asMap().map((_, l10n) => MapEntry(l10n.localeName, l10n.pidIdCardTitle)),
      subtitle: _subtitleMapper.map(input),
      logoImage: WalletAssets.logo_card_rijksoverheid,
      holoImage: WalletAssets.svg_rijks_card_holo,
      backgroundImage: WalletAssets.svg_rijks_card_bg_light,
      theme: CardFrontTheme.light,
    );
  }

  CardFront _getAddressCardFront(List<AppLocalizations> l10ns, Attestation input) {
    return CardFront(
      title: l10ns.asMap().map((_, l10n) => MapEntry(l10n.localeName, l10n.pidAddressCardTitle)),
      subtitle: _subtitleMapper.map(input),
      logoImage: WalletAssets.logo_card_rijksoverheid,
      holoImage: WalletAssets.svg_rijks_card_holo,
      backgroundImage: WalletAssets.svg_rijks_card_bg_dark,
      theme: CardFrontTheme.dark,
    );
  }

  /// Hacky way to figure out if the card is a renewed license, this will be removed once we properly implement
  /// a way to get the [CardFront]s through the core.
  bool isRenewedLicense(Attestation input) => input.attributes
      .map((attribute) => attribute.value)
      .whereType<AttributeValue_String>()
      .any((value) => value.value.contains('C1'));
}

// region CardFronts

final _kMockDiplomaCardFront = CardFront(
  title: 'BSc. Diploma'.untranslated,
  info: 'Dienst Uitvoerend Onderwijs'.untranslated,
  logoImage: WalletAssets.logo_card_rijksoverheid,
  backgroundImage: WalletAssets.image_bg_diploma,
  theme: CardFrontTheme.dark,
);

final _kMockMasterDiplomaCardFront = CardFront(
  title: 'MSc. Diploma'.untranslated,
  info: 'Dienst Uitvoerend Onderwijs'.untranslated,
  logoImage: WalletAssets.logo_card_rijksoverheid,
  backgroundImage: WalletAssets.image_bg_diploma,
  theme: CardFrontTheme.dark,
);

final _kMockDrivingLicenseCardFront = CardFront(
  title: 'Rijbewijs'.untranslated,
  subtitle: 'Categorie AM, B, BE'.untranslated,
  logoImage: WalletAssets.logo_nl_driving_license,
  backgroundImage: WalletAssets.image_bg_nl_driving_license,
  theme: CardFrontTheme.light,
);

final _kMockDrivingLicenseRenewedCardFront = CardFront(
  title: 'Rijbewijs'.untranslated,
  subtitle: 'Categorie AM, B, C1, BE'.untranslated,
  logoImage: WalletAssets.logo_nl_driving_license,
  backgroundImage: WalletAssets.image_bg_nl_driving_license,
  theme: CardFrontTheme.light,
);

final _kMockHealthInsuranceCardFront = CardFront(
  title: 'European Health Insurance Card'.untranslated,
  subtitle: 'Zorgverzekeraar Z'.untranslated,
  logoImage: WalletAssets.logo_nl_health_insurance,
  backgroundImage: WalletAssets.image_bg_health_insurance,
  theme: CardFrontTheme.dark,
);

final _kMockVOGCardFront = CardFront(
  title: 'Verklaring Omtrent het Gedrag'.untranslated,
  info: 'Justis'.untranslated,
  logoImage: WalletAssets.logo_card_rijksoverheid,
  backgroundImage: WalletAssets.image_bg_diploma,
  theme: CardFrontTheme.dark,
);
// endregion
