import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/verification/get_verification_request_usecase.dart';
import '../../../wallet_constants.dart';
import '../model/organization.dart';
import '../model/verification_request.dart';

part 'verification_event.dart';
part 'verification_state.dart';

class VerificationBloc extends Bloc<VerificationEvent, VerificationState> {
  final GetVerificationRequestUseCase getVerificationRequestUseCase;

  VerificationBloc(this.getVerificationRequestUseCase) : super(VerificationInitial()) {
    on<VerificationLoadRequested>(_onVerificationLoadRequested);
    on<VerificationOrganizationApproved>(_onVerificationOrganizationApproved);
    on<VerificationShareRequestedAttributesApproved>(_onVerificationShareRequestedAttributesApproved);
    on<VerificationPinConfirmed>(_onVerificationPinConfirmed);
    on<VerificationBackPressed>(_onVerificationBackPressed);
    on<VerificationStopRequested>(_onVerificationStopRequested);
  }

  void _onVerificationLoadRequested(VerificationLoadRequested event, emit) async {
    if (state is VerificationInitial) {
      try {
        emit(VerificationLoadInProgress());
        final request = await getVerificationRequestUseCase.invoke(event.sessionId);
        emit(VerificationCheckOrganization(request));
      } catch (ex, stack) {
        Fimber.e('Failed to load VerificationRequest for id ${event.sessionId}', ex: ex, stacktrace: stack);
        emit(VerificationGenericError());
      }
    }
  }

  void _onVerificationOrganizationApproved(VerificationOrganizationApproved event, emit) {
    final state = this.state;
    if (state is VerificationCheckOrganization) {
      if (state.request.hasMissingAttributes) {
        emit(VerificationMissingAttributes(state.request));
      } else {
        emit(VerificationConfirmDataAttributes(state.request));
      }
    }
  }

  void _onVerificationShareRequestedAttributesApproved(VerificationShareRequestedAttributesApproved event, emit) {
    final state = this.state;
    if (state is VerificationConfirmDataAttributes) emit(VerificationConfirmPin(state.request));
  }

  void _onVerificationPinConfirmed(VerificationPinConfirmed event, emit) async {
    final state = this.state;
    if (state is VerificationConfirmPin) {
      emit(VerificationLoadInProgress());
      await Future.delayed(kDefaultMockDelay);
      emit(VerificationSuccess(state.request));
    }
  }

  void _onVerificationBackPressed(VerificationBackPressed event, emit) {
    final state = this.state;
    if (state.canGoBack) {
      if (state is VerificationConfirmDataAttributes) {
        emit(VerificationCheckOrganization(state.request, afterBackPressed: true));
      } else if (state is VerificationMissingAttributes) {
        emit(VerificationCheckOrganization(state.request, afterBackPressed: true));
      } else if (state is VerificationConfirmPin) {
        emit(VerificationConfirmDataAttributes(state.request, afterBackPressed: true));
      }
    }
  }

  void _onVerificationStopRequested(VerificationStopRequested event, emit) async {
    emit(VerificationLoadInProgress());
    await Future.delayed(kDefaultMockDelay);
    emit(const VerificationStopped());
  }
}
