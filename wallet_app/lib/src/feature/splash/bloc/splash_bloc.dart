import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/app/check_is_app_initialized_usecase.dart';
import '../../../domain/usecase/wallet/is_wallet_initialized_with_pid_usecase.dart';
import '../../../wallet_constants.dart';

part 'splash_event.dart';

part 'splash_state.dart';

class SplashBloc extends Bloc<SplashEvent, SplashState> {
  final IsWalletInitializedUseCase isWalletInitializedUseCase;
  final IsWalletInitializedWithPidUseCase isWalletInitializedWithPidUseCase;

  SplashBloc(this.isWalletInitializedUseCase, this.isWalletInitializedWithPidUseCase, {initOnCreate = true})
      : super(SplashInitial()) {
    on<InitSplashEvent>((event, emit) async {
      await Future.delayed(kDefaultMockDelay);
      try {
        final isInitialized = await isWalletInitializedUseCase.invoke();
        final containsPid = await isWalletInitializedWithPidUseCase.invoke();
        emit(SplashLoaded(isRegistered: isInitialized, hasPid: containsPid));
      } catch (ex) {
        Fimber.e('Failed to check wallet initialization state', ex: ex);
      }
    });

    if (initOnCreate) add(const InitSplashEvent());
  }
}
