import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/verification/get_verification_request_usecase.dart';
import '../../../domain/usecase/wallet/get_requested_attributes_from_wallet_usecase.dart';
import '../../../wallet_constants.dart';
import '../model/organization.dart';
import '../model/verification_flow.dart';

part 'verification_event.dart';
part 'verification_state.dart';

class VerificationBloc extends Bloc<VerificationEvent, VerificationState> {
  final GetVerificationRequestUseCase getVerificationRequestUseCase;
  final GetRequestedAttributesFromWalletUseCase getRequestedAttributesFromWalletUseCase;

  VerificationBloc(this.getVerificationRequestUseCase, this.getRequestedAttributesFromWalletUseCase)
      : super(VerificationInitial()) {
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
        emit(
          VerificationCheckOrganization(
            VerificationFlow(
              id: request.id,
              organization: request.organization,
              requestedDataAttributes:
                  await getRequestedAttributesFromWalletUseCase.invoke(request.requestedAttributes),
              policy: request.policy,
            ),
          ),
        );
      } catch (ex, stack) {
        Fimber.e('Failed to load VerificationRequest for id ${event.sessionId}', ex: ex, stacktrace: stack);
        emit(VerificationGenericError());
      }
    }
  }

  void _onVerificationOrganizationApproved(VerificationOrganizationApproved event, emit) {
    final state = this.state;
    if (state is VerificationCheckOrganization) {
      if (state.flow.hasMissingAttributes) {
        emit(VerificationMissingAttributes(state.flow));
      } else {
        emit(VerificationConfirmDataAttributes(state.flow));
      }
    }
  }

  void _onVerificationShareRequestedAttributesApproved(VerificationShareRequestedAttributesApproved event, emit) {
    final state = this.state;
    if (state is VerificationConfirmDataAttributes) emit(VerificationConfirmPin(state.flow));
  }

  void _onVerificationPinConfirmed(VerificationPinConfirmed event, emit) async {
    final state = this.state;
    if (state is VerificationConfirmPin) {
      emit(VerificationLoadInProgress());
      await Future.delayed(kDefaultMockDelay);
      emit(VerificationSuccess(state.flow));
    }
  }

  void _onVerificationBackPressed(VerificationBackPressed event, emit) {
    final state = this.state;
    if (state.canGoBack) {
      if (state is VerificationConfirmDataAttributes) {
        emit(VerificationCheckOrganization(state.flow, afterBackPressed: true));
      } else if (state is VerificationMissingAttributes) {
        emit(VerificationCheckOrganization(state.flow, afterBackPressed: true));
      } else if (state is VerificationConfirmPin) {
        emit(VerificationConfirmDataAttributes(state.flow, afterBackPressed: true));
      }
    }
  }

  void _onVerificationStopRequested(VerificationStopRequested event, emit) async {
    emit(VerificationLoadInProgress());
    await Future.delayed(kDefaultMockDelay);
    emit(const VerificationStopped());
  }
}
