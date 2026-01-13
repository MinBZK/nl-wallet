import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/revocation/get_registration_revocation_code_usecase.dart';
import '../../../domain/usecase/revocation/set_revocation_code_saved_usecase.dart';
import '../../../util/cast_util.dart';

part 'revocation_code_event.dart';
part 'revocation_code_state.dart';

class RevocationCodeBloc extends Bloc<RevocationCodeEvent, RevocationCodeState> {
  final SetRevocationCodeSavedUseCase _setRevocationCodeSavedUseCase;
  final GetRegistrationRevocationCodeUseCase _getRegistrationRevocationCode;

  RevocationCodeBloc(
    this._setRevocationCodeSavedUseCase,
    this._getRegistrationRevocationCode,
  ) : super(const RevocationCodeInitial()) {
    on<RevocationCodeLoadTriggered>(_onLoadTriggered);
    on<RevocationCodeContinuePressed>(_onContinuePressed);
  }

  FutureOr<void> _onLoadTriggered(RevocationCodeLoadTriggered event, Emitter<RevocationCodeState> emit) async {
    final result = await _getRegistrationRevocationCode.invoke();
    await result.process(
      onSuccess: (revocationCode) => emit(RevocationCodeLoadSuccess(revocationCode)),
      onError: (error) => throw StateError('Failed to load revocation code'),
    );
  }

  FutureOr<void> _onContinuePressed(RevocationCodeContinuePressed event, Emitter<RevocationCodeState> emit) async {
    final revocationCode = tryCast<RevocationCodeLoadSuccess>(state)?.revocationCode;
    assert(revocationCode != null, 'RevocationCode should have been presented to the user');

    // Persist revocation code saved flag
    final result = await _setRevocationCodeSavedUseCase.invoke(saved: true);
    // Since there is no real error state, and setting the flag is not crucial we only log the error result
    if (result.hasError) Fimber.e('Failed to persist revocation code saved flag', ex: result.error);

    // Emit success, which should navigate the user to the next screen
    emit(RevocationCodeSaveSuccess(revocationCode!));
  }
}
