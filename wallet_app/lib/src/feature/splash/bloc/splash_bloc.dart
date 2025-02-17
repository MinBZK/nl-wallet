import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../environment.dart';
import '../../../domain/usecase/app/check_is_app_initialized_usecase.dart';
import '../../../domain/usecase/wallet/is_wallet_initialized_with_pid_usecase.dart';
import '../../../wallet_constants.dart';

part 'splash_event.dart';
part 'splash_state.dart';

class SplashBloc extends Bloc<SplashEvent, SplashState> {
  final IsWalletInitializedUseCase isWalletInitializedUseCase;
  final IsWalletInitializedWithPidUseCase isWalletInitializedWithPidUseCase;

  SplashBloc(this.isWalletInitializedUseCase, this.isWalletInitializedWithPidUseCase) : super(SplashInitial()) {
    on<InitSplashEvent>(_initApp);
  }

  Future<void> _initApp(InitSplashEvent event, Emitter<SplashState> emit) async {
    final skipDelay = Environment.isTest || !Environment.mockRepositories;
    await Future.delayed(skipDelay ? Duration.zero : kDefaultMockDelay);
    final isInitialized = await isWalletInitializedUseCase.invoke();
    final containsPid = await isWalletInitializedWithPidUseCase.invoke();
    emit(SplashLoaded(isRegistered: isInitialized, hasPid: containsPid));
  }
}
