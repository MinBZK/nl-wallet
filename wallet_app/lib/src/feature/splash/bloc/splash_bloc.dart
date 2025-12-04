import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../environment.dart';
import '../../../domain/model/wallet_state.dart';
import '../../../domain/usecase/wallet/get_wallet_state_usecase.dart';
import '../../../util/cast_util.dart';
import '../../../wallet_constants.dart';

part 'splash_event.dart';
part 'splash_state.dart';

class SplashBloc extends Bloc<SplashEvent, SplashState> {
  final GetWalletStateUseCase _getWalletStateUseCase;

  SplashBloc(this._getWalletStateUseCase) : super(SplashInitial()) {
    on<InitSplashEvent>(_initApp);
  }

  Future<void> _initApp(
    InitSplashEvent event,
    Emitter<SplashState> emit,
  ) async {
    final skipDelay = Environment.isTest || !Environment.mockRepositories;
    await Future.delayed(skipDelay ? Duration.zero : kDefaultMockDelay);

    final state = await _getWalletStateUseCase.invoke();
    final unlockedState = tryCast<WalletStateLocked>(state)?.substate ?? state;
    switch (unlockedState) {
      case WalletStateLocked():
        throw StateError('UnlockedState state should never be $unlockedState');
      case WalletStateUnregistered():
        emit(const SplashLoaded(.onboarding));
      case WalletStateEmpty():
        emit(const SplashLoaded(.pidRetrieval));
      case WalletStateTransferPossible():
        emit(const SplashLoaded(.transfer));
      case WalletStateTransferring():
        if (unlockedState.role == .target) {
          emit(const SplashLoaded(.transfer));
        } else {
          /// Transfer will be cancelled by [WalletTransferEventListener]
          emit(const SplashLoaded(.dashboard));
        }
      case WalletStateBlocked():
        emit(const SplashLoaded(.blocked));
      case WalletStateReady():
      case WalletStateInDisclosureFlow():
      case WalletStateInIssuanceFlow():
      case WalletStateInPinChangeFlow():
      case WalletStateInPinRecoveryFlow():
        emit(const SplashLoaded(.dashboard));
    }
  }
}
