import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/verification/get_verifier_policy_usecase.dart';
import '../../verification/model/verifier_policy.dart';

part 'verifier_policy_event.dart';
part 'verifier_policy_state.dart';

class VerifierPolicyBloc extends Bloc<VerifierPolicyEvent, VerifierPolicyState> {
  final GetVerifierPolicyUseCase getVerifierPolicyUseCase;

  VerifierPolicyBloc(this.getVerifierPolicyUseCase) : super(VerifierPolicyInitial()) {
    on<VerifierPolicyLoadTriggered>(_onVerifierPolicyLoadTriggered);
  }

  FutureOr<void> _onVerifierPolicyLoadTriggered(VerifierPolicyLoadTriggered event, emit) async {
    emit(VerifierPolicyLoadInProgress());
    try {
      emit(VerifierPolicyLoadSuccess(await getVerifierPolicyUseCase.invoke(event.sessionId)));
    } catch (ex, stack) {
      Fimber.e('Failed to fetch verifier policy', ex: ex, stacktrace: stack);
      emit(VerifierPolicyLoadFailure(event.sessionId));
    }
  }
}
