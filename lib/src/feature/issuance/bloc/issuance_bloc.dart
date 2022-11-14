import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/issuance_response.dart';
import '../../../domain/usecase/issuance/get_issuance_response_usecase.dart';
import '../../../wallet_constants.dart';
import '../../verification/model/organization.dart';

part 'issuance_event.dart';
part 'issuance_state.dart';

class IssuanceBloc extends Bloc<IssuanceEvent, IssuanceState> {
  final GetIssuanceResponseUseCase getIssueResponseUseCase;

  IssuanceBloc(this.getIssueResponseUseCase) : super(IssuanceInitial()) {
    on<IssuanceLoadTriggered>(_onIssuanceLoadTriggered);
    on<IssuanceVerifierDeclined>(_onIssuanceVerifierDeclined);
    on<IssuanceVerifierApproved>(_onIssuanceVerifierApproved);
    on<IssuanceBackPressed>(_onIssuanceBackPressed);
  }

  FutureOr<void> _onIssuanceLoadTriggered(IssuanceLoadTriggered event, emit) async {
    emit(IssuanceLoadInProgress());
    await Future.delayed(kDefaultMockDelay);
    final response = await getIssueResponseUseCase.invoke(event.sessionId);
    emit(IssuanceCheckOrganization(response));
  }

  FutureOr<void> _onIssuanceVerifierDeclined(IssuanceVerifierDeclined event, emit) async {
    emit(IssuanceLoadInProgress());
    await Future.delayed(kDefaultMockDelay);
    emit(IssuanceStopped());
  }

  FutureOr<void> _onIssuanceVerifierApproved(IssuanceVerifierApproved event, emit) async {
    final state = this.state;
    if (state is! IssuanceCheckOrganization) throw UnsupportedError('Incorrect state to $state');
    emit(IssuanceProofIdentity(state.response));
  }

  FutureOr<void> _onIssuanceBackPressed(IssuanceBackPressed event, emit) async {
    final state = this.state;
    if (state.canGoBack) {
      if (state is IssuanceProofIdentity) emit(IssuanceCheckOrganization(state.response));
    }
  }
}
