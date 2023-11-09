import 'dart:async';

import 'package:fimber/fimber.dart';

import '../../../domain/model/policy/policy.dart';
import '../../../domain/usecase/disclosure/cancel_disclosure_usecase.dart';
import '../../../domain/usecase/disclosure/start_disclosure_usecase.dart';
import 'disclosure_bloc.dart';

class CoreDisclosureBloc extends DisclosureBloc {
  final StartDisclosureUseCase _startDisclosureUseCase;
  final CancelDisclosureUseCase _cancelDisclosureUseCase;

  StartDisclosureResult? _startDisclosureResult;
  StreamSubscription? _startDisclosureStreamSubscription;

  CoreDisclosureBloc(
    String disclosureUri,
    this._startDisclosureUseCase,
    this._cancelDisclosureUseCase,
  ) : super(initialState: DisclosureLoadInProgress()) {
    on<DisclosureUpdateState>((event, emit) => emit(event.state));
    on<DisclosureStopRequested>(_onStopRequested);
    on<DisclosureBackPressed>(_onBackPressed);
    on<DisclosureOrganizationApproved>(_onOrganizationApproved);
    on<DisclosureShareRequestedAttributesApproved>(_onShareRequestApproved);
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
            '',
            true,
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
      emit(
        DisclosureCheckOrganization(
          state.relyingParty,
          '',
          true,
          afterBackPressed: true,
        ),
      );
    } else if (state is DisclosureMissingAttributes) {
      emit(
        DisclosureCheckOrganization(
          state.relyingParty,
          '',
          true,
          afterBackPressed: true,
        ),
      );
    } else if (state is DisclosureConfirmPin) {
      assert(_startDisclosureResult is StartDisclosureReadyToDisclose, 'Invalid state');
      emit(
        DisclosureConfirmDataAttributes(
          _startDisclosureResult!.relyingParty,
          (_startDisclosureResult as StartDisclosureReadyToDisclose).requestedAttributes,
          kMockPolicy,
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
            kMockPolicy,
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

  void _onShareRequestApproved(DisclosureShareRequestedAttributesApproved event, emit) {
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

// TODO: Replace with non-mock policy coming from the core.
const kMockPolicy = Policy(
  storageDuration: Duration(days: 1),
  dataIsShared: false,
  dataIsSignature: false,
  dataContainsSingleViewProfilePhoto: false,
  deletionCanBeRequested: false,
  privacyPolicyUrl: '',
);
