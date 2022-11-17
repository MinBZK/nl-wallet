import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/data_attribute.dart';
import '../../../domain/model/issuance_response.dart';
import '../../../domain/usecase/issuance/get_issuance_response_usecase.dart';
import '../../../wallet_constants.dart';
import '../../verification/model/organization.dart';

part 'issuance_event.dart';
part 'issuance_state.dart';

class IssuanceBloc extends Bloc<IssuanceEvent, IssuanceState> {
  final GetIssuanceResponseUseCase getIssuanceResponseUseCase;

  IssuanceBloc(this.getIssuanceResponseUseCase) : super(IssuanceInitial()) {
    on<IssuanceLoadTriggered>(_onIssuanceLoadTriggered);
    on<IssuanceBackPressed>(_onIssuanceBackPressed);
    on<IssuanceOrganizationDeclined>(_onIssuanceOrganizationDeclined);
    on<IssuanceOrganizationApproved>(_onIssuanceOrganizationApproved);
    on<IssuanceShareRequestedAttributesDeclined>(_onIssuanceShareRequestedAttributesDeclined);
    on<IssuanceShareRequestedAttributesApproved>(_onIssuanceShareRequestedAttributesApproved);
    on<IssuancePinConfirmed>(_onIssuancePinConfirmed);
    on<IssuanceStopRequested>(_onIssuanceStopRequested);
  }

  FutureOr<void> _onIssuanceBackPressed(IssuanceBackPressed event, emit) async {
    final state = this.state;
    if (state.canGoBack) {
      if (state is IssuanceProofIdentity) emit(IssuanceCheckOrganization(state.response, afterBackPressed: true));
      if (state is IssuanceProvidePin) emit(IssuanceProofIdentity(state.response, afterBackPressed: true));
    }
  }

  FutureOr<void> _onIssuanceLoadTriggered(IssuanceLoadTriggered event, emit) async {
    emit(IssuanceLoadInProgress());
    await Future.delayed(kDefaultMockDelay);
    final response = await getIssuanceResponseUseCase.invoke(event.sessionId);
    emit(IssuanceCheckOrganization(response));
  }

  FutureOr<void> _onIssuanceOrganizationDeclined(IssuanceOrganizationDeclined event, emit) async {
    emit(IssuanceLoadInProgress());
    await Future.delayed(kDefaultMockDelay);
    emit(IssuanceStopped());
  }

  FutureOr<void> _onIssuanceOrganizationApproved(IssuanceOrganizationApproved event, emit) async {
    final state = this.state;
    if (state is! IssuanceCheckOrganization) throw UnsupportedError('Incorrect state to $state');
    emit(IssuanceProofIdentity(state.response));
  }

  FutureOr<void> _onIssuanceShareRequestedAttributesDeclined(
      IssuanceShareRequestedAttributesDeclined event, emit) async {
    emit(IssuanceLoadInProgress());
    await Future.delayed(kDefaultMockDelay);
    emit(IssuanceStopped());
  }

  FutureOr<void> _onIssuanceShareRequestedAttributesApproved(
      IssuanceShareRequestedAttributesApproved event, emit) async {
    final state = this.state;
    if (state is! IssuanceProofIdentity) throw UnsupportedError('Incorrect state to $state');
    emit(IssuanceProvidePin(state.response));
  }

  FutureOr<void> _onIssuancePinConfirmed(IssuancePinConfirmed event, emit) async {
    final state = this.state;
    if (state is! IssuanceProvidePin) throw UnsupportedError('Incorrect state to $state');
    emit(IssuanceCheckDataOffering(state.response));
  }

  FutureOr<void> _onIssuanceStopRequested(IssuanceStopRequested event, emit) async {
    emit(IssuanceLoadInProgress());
    await Future.delayed(kDefaultMockDelay);
    emit(IssuanceStopped());
  }
}
