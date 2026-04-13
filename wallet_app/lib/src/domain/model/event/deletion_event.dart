part of './wallet_event.dart';

class DeletionEvent extends WalletEvent {
  /// The wallet card that was deleted.
  final WalletCard card;

  @override
  List<DataAttribute> get sharedAttributes => card.attributes;

  const DeletionEvent({
    required super.dateTime,
    required super.status,
    required this.card,
  });

  @override
  List<Object?> get props => [dateTime, status, card];
}
