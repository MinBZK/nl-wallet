import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/verification/get_verification_request_usecase.dart';
import '../model/verification_request.dart';

part 'verification_event.dart';
part 'verification_state.dart';

class VerificationBloc extends Bloc<VerificationEvent, VerificationState> {
  final GetVerificationRequestUseCase getVerificationRequestUseCase;

  VerificationBloc(this.getVerificationRequestUseCase) : super(VerificationInitial()) {
    on<VerificationLoadRequested>(_onVerificationLoadRequested);
  }

  void _onVerificationLoadRequested(VerificationLoadRequested event, emit) async {
    if (state is VerificationInitial) {
      try {
        emit(VerificationLoadInProgress());
        final request = await getVerificationRequestUseCase.invoke(event.sessionId);
        emit(VerificationLoadSuccess(request: request));
      } catch (ex, stack) {
        Fimber.e('Failed to load VerificationRequest for id ${event.sessionId}', ex: ex, stacktrace: stack);
        emit(VerificationLoadFailure());
      }
    }
  }
}
