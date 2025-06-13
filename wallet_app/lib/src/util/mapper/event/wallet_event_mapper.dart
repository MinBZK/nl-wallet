import 'package:wallet_core/core.dart' as core show DisclosureStatus, DisclosureType, Organization, WalletEvent;
import 'package:wallet_core/core.dart' hide DisclosureStatus, DisclosureType, Organization, WalletEvent;

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/event/wallet_event.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../mapper.dart';

class WalletEventMapper extends Mapper<core.WalletEvent, WalletEvent> {
  final Mapper<core.Organization, Organization> _relyingPartyMapper;
  final Mapper<RequestPolicy, Policy> _policyMapper;
  final Mapper<AttestationPresentation, WalletCard> _cardMapper;
  final Mapper<List<LocalizedString>, LocalizedText> _localizedStringMapper;
  final Mapper<core.DisclosureType, DisclosureType> _disclosureTypeMapper;

  WalletEventMapper(
    this._cardMapper,
    this._relyingPartyMapper,
    this._policyMapper,
    this._localizedStringMapper,
    this._disclosureTypeMapper,
  );

  @override
  WalletEvent map(core.WalletEvent input) {
    return switch (input) {
      WalletEvent_Disclosure() => WalletEvent.disclosure(
          dateTime: DateTime.parse(input.dateTime).toLocal(),
          relyingParty: _relyingPartyMapper.map(input.relyingParty),
          purpose: _localizedStringMapper.map(input.purpose),
          cards: _cardMapper.mapList(input.sharedAttestations ?? []),
          policy: _policyMapper.map(input.requestPolicy),
          status: _resolveInteractionStatus(input.status),
          type: _disclosureTypeMapper.map(input.typ),
        ),
      WalletEvent_Issuance() => WalletEvent.issuance(
          dateTime: DateTime.parse(input.dateTime).toLocal(),
          status: EventStatus.success,
          card: _cardMapper.map(input.attestation),
          renewed: input.renewed,
        ),
    };
  }

  EventStatus _resolveInteractionStatus(core.DisclosureStatus status) {
    return switch (status) {
      core.DisclosureStatus.Success => EventStatus.success,
      core.DisclosureStatus.Cancelled => EventStatus.cancelled,
      core.DisclosureStatus.Error => EventStatus.error,
    };
  }
}
