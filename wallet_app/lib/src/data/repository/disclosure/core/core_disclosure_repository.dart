import 'package:rxdart/rxdart.dart';

import '../../../../../bridge_generated.dart';
import '../../../../domain/model/attribute/requested_attribute.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../util/mapper/locale_mapper.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../../../store/active_locale_provider.dart';
import '../../organization/organization_repository.dart';
import '../disclosure_repository.dart';

class CoreDisclosureRepository implements DisclosureRepository {
  final TypedWalletCore _walletCore;

  final LocaleMapper<RequestedCard, WalletCard> _cardMapper;
  final LocaleMapper<MissingAttribute, RequestedAttribute> _missingAttributeMapper;
  final Mapper<RelyingParty, Organization> _relyingPartyMapper;
  final ActiveLocaleProvider _localeProvider;

  CoreDisclosureRepository(
    this._walletCore,
    this._cardMapper,
    this._relyingPartyMapper,
    this._missingAttributeMapper,
    this._localeProvider,
  );

  @override
  Stream<StartDisclosureResult> startDisclosure(Uri disclosureUri) {
    return CombineLatestStream.combine2(
        Stream.fromFuture(_walletCore.startDisclosure(disclosureUri)), _localeProvider.observe(), (result, locale) {
      return result.map(
        request: (value) {
          final cards = _cardMapper.mapList(locale, value.requestedCards);
          final requestedAttributes = cards.asMap().map((key, value) => MapEntry(value, value.attributes));
          final relyingParty = _relyingPartyMapper.map(value.relyingParty);
          return StartDisclosureReadyToDisclose(relyingParty, requestedAttributes);
        },
        requestAttributesMissing: (value) {
          final relyingParty = _relyingPartyMapper.map(value.relyingParty);
          final missingAttributes = _missingAttributeMapper.mapList(locale, value.missingAttributes);
          return StartDisclosureMissingAttributes(relyingParty, missingAttributes);
        },
      );
    });
  }

  @override
  Future<void> cancelDisclosure() => _walletCore.cancelDisclosure();

  @override
  Future<WalletInstructionResult> acceptDisclosure(String pin) => _walletCore.acceptDisclosure(pin);
}
