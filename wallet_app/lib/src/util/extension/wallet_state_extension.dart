import '../../domain/model/wallet_state.dart';
import '../cast_util.dart';

extension WalletStateExtension on WalletState {
  /// Unwraps the [WalletState] from a [WalletStateLocked].
  ///
  /// If the wallet is locked, it returns the underlying `substate`.
  /// Otherwise, it returns the current state.
  WalletState get unlockedState => tryCast<WalletStateLocked>(this)?.substate ?? this;
}
