import 'package:wallet_core/core.dart' as core show DisclosureStatus, DisclosureType, Organization, WalletEvent;
import 'package:wallet_core/core.dart' hide DisclosureStatus, DisclosureType, Organization, WalletEvent;

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/event/wallet_event.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/wallet_card.dart';
import '../mapper.dart';

class WalletEventMapper extends Mapper<core.WalletEvent, WalletEvent> {
  final Mapper<core.Organization, Organization> _relyingPartyMapper;
  final Mapper<RequestPolicy, Policy> _policyMapper;
  final Mapper<Card, WalletCard> _cardMapper;
  final Mapper<DisclosureCard, WalletCard> _disclosureCardMapper;
  final Mapper<List<LocalizedString>, LocalizedText> _localizedStringMapper;
  final Mapper<core.DisclosureType, DisclosureType> _disclosureTypeMapper;

  WalletEventMapper(
    this._cardMapper,
    this._disclosureCardMapper,
    this._relyingPartyMapper,
    this._policyMapper,
    this._localizedStringMapper,
    this._disclosureTypeMapper,
  );

  @override
  WalletEvent map(core.WalletEvent input) {
    return input.map(
      disclosure: (disclosure) {
        final cards = _disclosureCardMapper.mapList(disclosure.requestedCards ?? []);
        return WalletEvent.disclosure(
          dateTime: DateTime.parse(disclosure.dateTime).toLocal(),
          relyingParty: _relyingPartyMapper.map(disclosure.relyingParty),
          purpose: _localizedStringMapper.map(disclosure.purpose),
          cards: cards,
          policy: _policyMapper.map(disclosure.requestPolicy),
          status: _resolveInteractionStatus(disclosure.status),
          type: _disclosureTypeMapper.map(disclosure.type),
        );
      },
      issuance: (issuance) {
        final card = _cardMapper.map(issuance.card);
        return WalletEvent.issuance(
          dateTime: DateTime.parse(issuance.dateTime).toLocal(),
          status: EventStatus.success,
          card: card,
        );
      },
    );
  }

  EventStatus _resolveInteractionStatus(core.DisclosureStatus status) {
    return switch (status) {
      core.DisclosureStatus.Success => EventStatus.success,
      core.DisclosureStatus.Cancelled => EventStatus.cancelled,
      core.DisclosureStatus.Error => EventStatus.error,
    };
  }
}
