import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:fimber/fimber.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../domain/usecase/disclosure/cancel_disclosure_usecase.dart';
import '../../../domain/usecase/disclosure/start_disclosure_usecase.dart';
import '../../report_issue/report_issue_screen.dart';
import '../model/disclosure_flow.dart';

part 'disclosure_event.dart';
part 'disclosure_state.dart';

class DisclosureBloc extends Bloc<DisclosureEvent, DisclosureState> {
  final StartDisclosureUseCase _startDisclosureUseCase;
  final CancelDisclosureUseCase _cancelDisclosureUseCase;

  StartDisclosureResult? _startDisclosureResult;
  StreamSubscription? _startDisclosureStreamSubscription;

  DisclosureBloc(
    String disclosureUri,
    this._startDisclosureUseCase,
    this._cancelDisclosureUseCase,
  ) : super(DisclosureLoadInProgress()) {
    on<DisclosureUpdateState>((event, emit) => emit(event.state));
    on<DisclosureStopRequested>(_onStopRequested);
    on<DisclosureBackPressed>(_onBackPressed);
    on<DisclosureOrganizationApproved>(_onOrganizationApproved);
    on<DisclosureShareRequestedAttributesApproved>(_onShareRequestedAttributesApproved);
    on<DisclosurePinConfirmed>(_onPinConfirmed);

    _initDisclosure(disclosureUri);
  }

  void _initDisclosure(String disclosureUri) async {
    try {
      _startDisclosureResult = await _startDisclosureUseCase.invoke(disclosureUri);
      add(
        DisclosureUpdateState(
          DisclosureCheckOrganization(
            _startDisclosureResult!.relyingParty,
            _startDisclosureResult!.requestPurpose,
            _startDisclosureResult!.isFirstInteractionWithOrganization,
          ),
        ),
      );
    } catch (ex) {
      Fimber.e('Failed to start disclosure', ex: ex);
      add(DisclosureUpdateState(DisclosureGenericError()));
    }
  }

  void _onStopRequested(DisclosureStopRequested event, emit) async {
    try {
      emit(DisclosureLoadInProgress());
      await _cancelDisclosureUseCase.invoke();
    } catch (ex) {
      Fimber.e('Failed to explicitly cancel disclosure flow', ex: ex);
    } finally {
      emit(const DisclosureStopped());
    }
  }

  void _onBackPressed(DisclosureBackPressed event, emit) async {
    final state = this.state;
    if (state is DisclosureConfirmDataAttributes) {
      assert(_startDisclosureResult != null, 'StartDisclosureResult should always be available at this stage');
      emit(
        DisclosureCheckOrganization(
          state.relyingParty,
          _startDisclosureResult!.requestPurpose,
          _startDisclosureResult?.isFirstInteractionWithOrganization == true,
          afterBackPressed: true,
        ),
      );
    } else if (state is DisclosureMissingAttributes) {
      assert(_startDisclosureResult != null, 'StartDisclosureResult should always be available at this stage');
      emit(
        DisclosureCheckOrganization(
          state.relyingParty,
          _startDisclosureResult!.requestPurpose,
          _startDisclosureResult?.isFirstInteractionWithOrganization == true,
          afterBackPressed: true,
        ),
      );
    } else if (state is DisclosureConfirmPin) {
      assert(_startDisclosureResult is StartDisclosureReadyToDisclose, 'Invalid state');
      final result = _startDisclosureResult as StartDisclosureReadyToDisclose;
      emit(
        DisclosureConfirmDataAttributes(
          _startDisclosureResult!.relyingParty,
          result.requestedAttributes,
          result.policy,
          afterBackPressed: true,
        ),
      );
    }
  }

  void _onOrganizationApproved(DisclosureOrganizationApproved event, emit) async {
    final startDisclosureResult = _startDisclosureResult;
    switch (startDisclosureResult) {
      case null:
        throw UnsupportedError('Organization approved while in invalid state, i.e. no result available!');
      case StartDisclosureReadyToDisclose():
        emit(
          DisclosureConfirmDataAttributes(
            startDisclosureResult.relyingParty,
            startDisclosureResult.requestedAttributes,
            startDisclosureResult.policy,
          ),
        );
      case StartDisclosureMissingAttributes():
        emit(
          DisclosureMissingAttributes(
            startDisclosureResult.relyingParty,
            startDisclosureResult.missingAttributes,
          ),
        );
    }
  }

  void _onShareRequestedAttributesApproved(DisclosureShareRequestedAttributesApproved event, emit) {
    assert(_startDisclosureResult is StartDisclosureReadyToDisclose, 'Invalid data state to continue disclosing');
    assert(state is DisclosureConfirmDataAttributes, 'Invalid UI state to move to pin entry');
    if (state is DisclosureConfirmDataAttributes) emit(const DisclosureConfirmPin());
  }

  void _onPinConfirmed(DisclosurePinConfirmed event, emit) {
    assert(_startDisclosureResult != null, 'DisclosureResult should still be available after confirming the tx');
    emit(DisclosureSuccess(_startDisclosureResult!.relyingParty));
  }

  @override
  Future<void> close() {
    _startDisclosureStreamSubscription?.cancel();
    return super.close();
  }
}
