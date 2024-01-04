import 'package:collection/collection.dart';
import 'package:wallet_core/core.dart' as core show Organization;
import 'package:wallet_core/core.dart' hide Organization;

import '../../../data/repository/organization/organization_repository.dart';
import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../../domain/model/timeline/timeline_attribute.dart';
import '../../../domain/model/wallet_card.dart';
import '../mapper.dart';

class WalletEventMapper extends Mapper<WalletEvent, TimelineAttribute> {
  final Mapper<core.Organization, Organization> _relyingPartyMapper;
  final Mapper<RequestPolicy, Policy> _policyMapper;
  final Mapper<Card, WalletCard> _cardMapper;
  final Mapper<RequestedCard, WalletCard> _requestedCardMapper;
  final Mapper<List<LocalizedString>, LocalizedText> _localizedStringMapper;

  WalletEventMapper(
    this._cardMapper,
    this._requestedCardMapper,
    this._relyingPartyMapper,
    this._policyMapper,
    this._localizedStringMapper,
  );

  @override
  TimelineAttribute map(WalletEvent input) {
    return input.map(disclosure: (disclosure) {
      final cards = _requestedCardMapper.mapList(disclosure.requestedCards ?? []);
      return InteractionTimelineAttribute(
        dateTime: DateTime.parse(disclosure.dateTime),
        organization: _relyingPartyMapper.map(disclosure.relyingParty),
        status: _resolveInteractionStatus(disclosure.status),
        policy: _policyMapper.map(disclosure.requestPolicy),
        requestPurpose: _localizedStringMapper.map(disclosure.purpose),
        dataAttributes: cards.map((e) => e.attributes).flattened.toList(),
      );
    }, issuance: (issuance) {
      final card = _cardMapper.map(issuance.card);
      return OperationTimelineAttribute(
        dateTime: DateTime.parse(issuance.dateTime),
        organization: _relyingPartyMapper.map(issuance.issuer),
        status: OperationStatus.issued,
        cardTitle: card.front.title,
        dataAttributes: card.attributes,
      );
    });
  }

  InteractionStatus _resolveInteractionStatus(DisclosureStatus status) {
    return switch (status) {
      DisclosureStatus.Success => InteractionStatus.success,
      DisclosureStatus.Cancelled => InteractionStatus.rejected,
      DisclosureStatus.Error => InteractionStatus.failed,
    };
  }
}
