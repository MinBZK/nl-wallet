part of './wallet_event.dart';

class IssuanceEvent extends WalletEvent {
  /// The wallet card associated with this issuance event.
  final WalletCard card;

  /// Indicates issuance event type
  final IssuanceEventType eventType;

  @override
  List<DataAttribute> get sharedAttributes => card.attributes;

  const IssuanceEvent({
    required super.dateTime,
    required super.status,
    required this.card,
    required this.eventType,
  });

  @override
  List<Object?> get props => [dateTime, status, card, eventType];
}

enum IssuanceEventType {
  cardIssued,
  cardRenewed,
  cardStatusExpired,
  cardStatusCorrupted,
  cardStatusRevoked,
}
