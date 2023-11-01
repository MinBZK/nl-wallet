import '../../../../domain/model/attribute/missing_attribute.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../../../../wallet_core/wallet_core.dart' as core show MissingAttribute;
import '../../../../wallet_core/wallet_core.dart' hide MissingAttribute;
import '../../organization/organization_repository.dart';
import '../disclosure_repository.dart';

class CoreDisclosureRepository implements DisclosureRepository {
  final TypedWalletCore _walletCore;

  final Mapper<RequestedCard, WalletCard> _cardMapper;
  final Mapper<core.MissingAttribute, MissingAttribute> _missingAttributeMapper;
  final Mapper<RelyingParty, Organization> _relyingPartyMapper;

  CoreDisclosureRepository(
    this._walletCore,
    this._cardMapper,
    this._relyingPartyMapper,
    this._missingAttributeMapper,
  );

  @override
  Future<StartDisclosureResult> startDisclosure(Uri disclosureUri) async {
    final result = await _walletCore.startDisclosure(disclosureUri);
    return result.map(
      request: (value) {
        final cards = _cardMapper.mapList(value.requestedCards);
        final requestedAttributes = cards.asMap().map((key, value) => MapEntry(value, value.attributes));
        final relyingParty = _relyingPartyMapper.map(value.relyingParty);
        return StartDisclosureReadyToDisclose(relyingParty, requestedAttributes);
      },
      requestAttributesMissing: (value) {
        final relyingParty = _relyingPartyMapper.map(value.relyingParty);
        final missingAttributes = _missingAttributeMapper.mapList(value.missingAttributes);
        return StartDisclosureMissingAttributes(relyingParty, missingAttributes);
      },
    );
  }

  @override
  Future<void> cancelDisclosure() => _walletCore.cancelDisclosure();

  @override
  Future<WalletInstructionResult> acceptDisclosure(String pin) => _walletCore.acceptDisclosure(pin);
}
