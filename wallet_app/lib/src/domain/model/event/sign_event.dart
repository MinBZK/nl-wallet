part of './wallet_event.dart';

class SignEvent extends WalletEvent {
  final Organization relyingParty;
  final Policy policy;
  final Document document;

  @override
  List<DataAttribute> get sharedAttributes => [];

  const SignEvent({
    required super.dateTime,
    required super.status,
    required this.relyingParty,
    required this.policy,
    required this.document,
  });

  @override
  List<Object?> get props => [dateTime, status, relyingParty, policy, document];
}
