import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/app/check_is_app_initialized_usecase.dart';

part 'splash_event.dart';
part 'splash_state.dart';

class SplashBloc extends Bloc<SplashEvent, SplashState> {
  final CheckIsAppInitializedUseCase checkIsAppInitializedUseCase;

  SplashBloc(this.checkIsAppInitializedUseCase) : super(SplashInitial()) {
    on<InitSplashEvent>((event, emit) async {
      emit(SplashLoaded(await checkIsAppInitializedUseCase.isInitialized()));
    });

    //Initialize immediately when bloc is created.
    add(const InitSplashEvent());
  }
}
