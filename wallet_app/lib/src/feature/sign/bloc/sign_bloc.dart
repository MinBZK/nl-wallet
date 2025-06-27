import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/document.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/start_sign_result/start_sign_result.dart';
import '../../../domain/usecase/sign/reject_sign_agreement_usecase.dart';
import '../../../domain/usecase/sign/start_sign_usecase.dart';

part 'sign_event.dart';
part 'sign_state.dart';

class SignBloc extends Bloc<SignEvent, SignState> {
  final StartSignUseCase _startSignUseCase;
  final RejectSignAgreementUseCase _rejectSignAgreementUseCase;

  StartSignResult? _startSignResult;

  SignBloc(
    String signUri,
    this._startSignUseCase,
    this._rejectSignAgreementUseCase,
  ) : super(const SignLoadInProgress()) {
    on<SignOrganizationApproved>(_onOrganizationApproved);
    on<SignAgreementChecked>(_onAgreementChecked);
    on<SignAgreementApproved>(_onAgreementApproved);
    on<SignPinConfirmed>(_onPinConfirmed);
    on<SignStopRequested>(_onStopRequested);
    on<SignBackPressed>(_onBackPressed);
    on<SignUpdateState>((event, emit) => emit(event.state));

    _initSigning(signUri);
  }

  Future<void> _initSigning(String signUri) async {
    final startResult = await _startSignUseCase.invoke(signUri);
    _startSignResult = startResult.value;

    await startResult.process(
      onSuccess: (result) {
        add(
          SignUpdateState(
            SignCheckOrganization(
              relyingParty: result.relyingParty,
            ),
          ),
        );
      },
      onError: (error) => add(const SignUpdateState(SignError())),
    );
  }

  Future<void> _onOrganizationApproved(event, emit) async {
    assert(state is SignCheckOrganization, 'State should be SignCheckOrganization when the user approves');
    assert(_startSignResult != null, 'Can not approve organization when result is not available');
    try {
      emit(
        SignCheckAgreement(
          relyingParty: _startSignResult!.relyingParty,
          document: _startSignResult!.document,
          trustProvider: _startSignResult!.trustProvider,
        ),
      );
    } catch (ex) {
      Fimber.e('Failed to move to SignCheckAgreement state', ex: ex);
      emit(const SignError());
    }
  }

  Future<void> _onAgreementChecked(event, emit) async {
    assert(state is SignCheckAgreement, 'State should be SignCheckAgreement when the user checks the agreement');
    assert(_startSignResult != null, 'Can not check agreement when result is not available');
    assert(
      _startSignResult is StartSignReadyToSign,
      'Mock only supports flow where all attributes are available for singing',
    );
    try {
      final requestedCards = (_startSignResult! as StartSignReadyToSign).requestedCards;
      emit(
        SignConfirmAgreement(
          document: _startSignResult!.document,
          relyingParty: _startSignResult!.relyingParty,
          policy: _startSignResult!.policy,
          requestedAttributes: requestedCards.map((it) => it.attributes).flattenedToList,
          trustProvider: _startSignResult!.trustProvider,
        ),
      );
    } catch (ex) {
      Fimber.e('Failed to move to SignConfirmAgreement state', ex: ex);
      emit(const SignError());
    }
  }

  Future<void> _onAgreementApproved(event, emit) async {
    assert(state is SignConfirmAgreement, 'State should be SignConfirmAgreement when the user approves the agreement');
    assert(_startSignResult != null, 'Can not approve agreement when result is not available');
    try {
      emit(const SignConfirmPin());
    } catch (ex) {
      Fimber.e('Failed to move to SignConfirmPin state', ex: ex);
      emit(const SignError());
    }
  }

  Future<void> _onPinConfirmed(event, emit) async {
    assert(state is SignConfirmPin, 'State should be SignConfirmPin when the user confirms with pin');
    assert(_startSignResult != null, 'Can not confirm pin when result is not available');
    emit(const SignLoadInProgress());
    try {
      emit(SignSuccess(relyingParty: _startSignResult!.relyingParty));
    } catch (ex) {
      Fimber.e('Failed to move to SignSuccess state', ex: ex);
      emit(const SignError());
    }
  }

  Future<void> _onStopRequested(event, emit) async {
    assert(_startSignResult != null, 'Stop can only be requested after flow is loaded');
    final rejectResult = await _rejectSignAgreementUseCase.invoke();
    await rejectResult.process(
      onSuccess: (_) => emit(const SignStopped()),
      onError: (error) => emit(const SignError()),
    );
  }

  Future<void> _onBackPressed(event, emit) async {
    final state = this.state;
    if (state.canGoBack) {
      if (state is SignCheckAgreement) {
        emit(
          SignCheckOrganization(
            relyingParty: _startSignResult!.relyingParty,
            afterBackPressed: true,
          ),
        );
      } else if (state is SignConfirmAgreement) {
        emit(
          SignCheckAgreement(
            relyingParty: _startSignResult!.relyingParty,
            trustProvider: _startSignResult!.trustProvider,
            document: _startSignResult!.document,
            afterBackPressed: true,
          ),
        );
      } else if (state is SignConfirmPin) {
        final requestedCards = (_startSignResult! as StartSignReadyToSign).requestedCards;
        emit(
          SignConfirmAgreement(
            policy: _startSignResult!.policy,
            relyingParty: _startSignResult!.relyingParty,
            trustProvider: _startSignResult!.trustProvider,
            document: _startSignResult!.document,
            afterBackPressed: true,
            requestedAttributes: requestedCards.map((it) => it.attributes).flattenedToList,
          ),
        );
      }
    }
  }
}
