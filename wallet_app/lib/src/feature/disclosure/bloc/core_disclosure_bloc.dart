import 'dart:async';

import 'package:fimber/fimber.dart';

import '../../../domain/model/policy/policy.dart';
import '../../../domain/usecase/disclosure/cancel_disclosure_usecase.dart';
import '../../../domain/usecase/disclosure/start_disclosure_usecase.dart';
import 'disclosure_bloc.dart';

class CoreDisclosureBloc extends DisclosureBloc {
  final StartDisclosureUseCase _startDisclosureUseCase;
  final CancelDisclosureUseCase _cancelDisclosureUseCase;

  StartDisclosureResult? _lastStartDisclosureResult;
  StreamSubscription? _startDisclosureStreamSubscription;

  CoreDisclosureBloc(
    Uri disclosureUri,
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

  void _initDisclosure(Uri disclosureUri) async {
    _startDisclosureStreamSubscription?.cancel();
    _startDisclosureStreamSubscription = _startDisclosureUseCase.invoke(disclosureUri).listen(
      (result) {
        _lastStartDisclosureResult = result;
        if (state is DisclosureLoadInProgress) {
          add(
            DisclosureUpdateState(
              DisclosureCheckOrganization(
                result.relyingParty,
                '',
                true,
              ),
            ),
          );
        } else if (state is DisclosureConfirmDataAttributes || state is DisclosureMissingAttributes) {
          // Propagate potential translation update, by 'approving' again while in any of these two states the
          // correct state will be re-emitted with the updated translations from [_lastStartDisclosureResult].
          add(const DisclosureOrganizationApproved());
        }
      },
      onError: (error) {
        add(DisclosureUpdateState(DisclosureGenericError()));
      },
      cancelOnError: true,
    );
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
      assert(_lastStartDisclosureResult is StartDisclosureReadyToDisclose, 'Invalid state');
      emit(
        DisclosureConfirmDataAttributes(
          _lastStartDisclosureResult!.relyingParty,
          (_lastStartDisclosureResult as StartDisclosureReadyToDisclose).requestedAttributes,
          kMockPolicy,
          afterBackPressed: true,
        ),
      );
    }
  }

  void _onOrganizationApproved(DisclosureOrganizationApproved event, emit) async {
    final startDisclosureResult = _lastStartDisclosureResult;
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
    assert(_lastStartDisclosureResult is StartDisclosureReadyToDisclose, 'Invalid data state to continue disclosing');
    assert(state is DisclosureConfirmDataAttributes, 'Invalid UI state to move to pin entry');
    if (state is DisclosureConfirmDataAttributes) emit(const DisclosureConfirmPin());
  }

  void _onPinConfirmed(DisclosurePinConfirmed event, emit) {
    assert(_lastStartDisclosureResult != null, 'DisclosureResult should still be available after confirming the tx');
    emit(DisclosureSuccess(_lastStartDisclosureResult!.relyingParty));
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
