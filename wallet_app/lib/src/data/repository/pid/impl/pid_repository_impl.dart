import 'dart:async';

import 'package:rxdart/rxdart.dart';

import '../../../../../bridge_generated.dart';
import '../../../../wallet_core/typed_wallet_core.dart';
import '../pid_repository.dart';

class PidRepositoryImpl extends PidRepository {
  final StreamController<PidIssuanceStatus> _digidAuthStatusController = BehaviorSubject();
  final TypedWalletCore _walletCore;

  PidRepositoryImpl(this._walletCore);

  @override
  Future<String> getPidIssuanceUrl() => _walletCore.createPidIssuanceRedirectUri();

  @override
  void notifyPidIssuanceStateUpdate(PidIssuanceEvent? event) {
    event?.when(
      authenticating: () {
        _digidAuthStatusController.add(PidIssuanceAuthenticating());
      },
      success: (success) {
        _digidAuthStatusController.add(PidIssuanceSuccess(List.empty()));
        _digidAuthStatusController.add(PidIssuanceIdle());
      },
      error: (error) {
        _digidAuthStatusController.add(PidIssuanceError());
        _digidAuthStatusController.add(PidIssuanceIdle());
      },
    );
    if (event == null) _digidAuthStatusController.add(PidIssuanceIdle());
  }

  @override
  Stream<PidIssuanceStatus> observePidIssuanceStatus() => _digidAuthStatusController.stream;
}
