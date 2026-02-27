part of 'app_blocked_bloc.dart';

abstract class AppBlockedEvent extends Equatable {
  const AppBlockedEvent();

  @override
  List<Object?> get props => [];
}

class AppBlockedLoadTriggered extends AppBlockedEvent {
  /// Optional [RevocationReason] used to identify the 'blocked by user' state.
  /// This is necessary because the state is transient; once the user revokes the wallet
  /// and the wallet_core detects and exposes [RevocationReason.userRequest] the wallet
  /// is cleared, meaning the state is lost.
  final RevocationReason reason;

  const AppBlockedLoadTriggered({this.reason = RevocationReason.unknown});

  @override
  List<Object?> get props => [reason];
}
