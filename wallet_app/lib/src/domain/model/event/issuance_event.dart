part of './wallet_event.dart';

class IssuanceEvent extends WalletEvent {
  /// The wallet card associated with this issuance event.
  final WalletCard card;

  /// Indicates whether the card was renewed (true) or newly issued (false).
  final bool renewed;

  @override
  List<DataAttribute> get sharedAttributes => card.attributes;

  const IssuanceEvent({
    required super.dateTime,
    required super.status,
    required this.card,
    required this.renewed,
  });

  @override
  List<Object?> get props => [dateTime, status, card, renewed];
}
