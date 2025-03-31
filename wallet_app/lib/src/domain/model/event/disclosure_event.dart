part of './wallet_event.dart';

class DisclosureEvent extends WalletEvent {
  final Organization relyingParty;
  final LocalizedText purpose;
  final List<WalletCard> cards;
  final Policy policy;
  final DisclosureType type;

  const DisclosureEvent({
    required super.dateTime,
    required super.status,
    required this.relyingParty,
    required this.purpose,
    required this.cards,
    required this.policy,
    required this.type,
  });

  @override
  List<DataAttribute> get sharedAttributes => cards.expand((e) => e.attributes).toList(growable: false);

  bool get hasSharedAttributes => sharedAttributes.isNotEmpty;

  @override
  List<Object?> get props => [dateTime, status, relyingParty, purpose, cards, policy, type];
}
