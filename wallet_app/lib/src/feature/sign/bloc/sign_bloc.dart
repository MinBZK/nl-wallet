import 'package:collection/collection.dart';
import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/document.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/start_sign_result/start_sign_result.dart';
import '../../../domain/usecase/card/log_card_signing_usecase.dart';
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

  void _initSigning(String signUri) async {
    try {
      final result = _startSignResult = await _startSignUseCase.invoke(signUri);
      add(
        SignUpdateState(
          SignCheckOrganization(
            organization: result.relyingParty,
          ),
        ),
      );
    } catch (ex) {
      Fimber.e('Failed to start disclosure', ex: ex);
      add(const SignUpdateState(SignError()));
    }
  }

  void _onOrganizationApproved(event, emit) async {
    assert(state is SignCheckOrganization);
    assert(_startSignResult != null, 'Can not approve organization when result is not available');
    try {
      emit(
        SignCheckAgreement(
          organization: _startSignResult!.relyingParty,
          document: _startSignResult!.document,
          trustProvider: _startSignResult!.trustProvider,
        ),
      );
    } catch (ex) {
      Fimber.e('Failed to move to SignCheckAgreement state', ex: ex);
      emit(const SignError());
    }
  }

  void _onAgreementChecked(event, emit) async {
    assert(state is SignCheckAgreement);
    assert(_startSignResult != null, 'Can not check agreement when result is not available');
    assert(_startSignResult is StartSignReadyToSign,
        'Mock only supports flow where all attributes are available for singing');
    try {
      final requestedAttributes = (_startSignResult as StartSignReadyToSign).requestedAttributes;
      emit(
        SignConfirmAgreement(
          document: _startSignResult!.document,
          policy: _startSignResult!.policy,
          requestedAttributes: requestedAttributes.values.flattened.toList(),
          trustProvider: _startSignResult!.trustProvider,
        ),
      );
    } catch (ex) {
      Fimber.e('Failed to move to SignConfirmAgreement state', ex: ex);
      emit(const SignError());
    }
  }

  void _onAgreementApproved(event, emit) async {
    assert(state is SignConfirmAgreement);
    assert(_startSignResult != null, 'Can not approve agreement when result is not available');
    try {
      emit(const SignConfirmPin());
    } catch (ex) {
      Fimber.e('Failed to move to SignConfirmPin state', ex: ex);
      emit(const SignError());
    }
  }

  void _onPinConfirmed(event, emit) async {
    assert(state is SignConfirmPin);
    assert(_startSignResult != null, 'Can not confirm pin when result is not available');
    emit(const SignLoadInProgress());
    try {
      emit(SignSuccess(organization: _startSignResult!.relyingParty));
    } catch (ex) {
      Fimber.e('Failed to move to SignSuccess state', ex: ex);
      emit(const SignError());
    }
  }

  void _onStopRequested(event, emit) async {
    assert(_startSignResult != null, 'Stop can only be requested after flow is loaded');
    try {
      await _rejectSignAgreementUseCase.invoke();
      emit(const SignStopped());
    } catch (ex) {
      Fimber.e('Failed to move to SignStopped state', ex: ex);
      emit(const SignError());
    }
  }

  void _onBackPressed(event, emit) async {
    final state = this.state;
    if (state.canGoBack) {
      if (state is SignCheckAgreement) {
        emit(SignCheckOrganization(
          organization: _startSignResult!.relyingParty,
          afterBackPressed: true,
        ));
      } else if (state is SignConfirmAgreement) {
        emit(SignCheckAgreement(
          organization: _startSignResult!.relyingParty,
          trustProvider: _startSignResult!.trustProvider,
          document: _startSignResult!.document,
          afterBackPressed: true,
        ));
      } else if (state is SignConfirmPin) {
        final requestedAttributes = (_startSignResult as StartSignReadyToSign).requestedAttributes.values;
        emit(SignConfirmAgreement(
          policy: _startSignResult!.policy,
          trustProvider: _startSignResult!.trustProvider,
          document: _startSignResult!.document,
          afterBackPressed: true,
          requestedAttributes: requestedAttributes.flattened.toList(),
        ));
      }
    }
  }
}
