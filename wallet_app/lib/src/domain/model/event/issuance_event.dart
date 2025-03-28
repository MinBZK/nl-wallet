part of './wallet_event.dart';

class IssuanceEvent extends WalletEvent {
  final WalletCard card;

  @override
  List<DataAttribute> get sharedAttributes => card.attributes;

  const IssuanceEvent({
    required super.dateTime,
    required super.status,
    required this.card,
  });

  @override
  List<Object?> get props => [dateTime, status, card];
}
