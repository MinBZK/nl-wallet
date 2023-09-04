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
    event?.when(authenticating: () {
      _digidAuthStatusController.add(PidIssuanceStatus.authenticating);
    }, success: (success) {
      _digidAuthStatusController.add(PidIssuanceStatus.success);
      _digidAuthStatusController.add(PidIssuanceStatus.idle);
    }, error: (error) {
      _digidAuthStatusController.add(PidIssuanceStatus.error);
      _digidAuthStatusController.add(PidIssuanceStatus.idle);
    });
    if (event == null) _digidAuthStatusController.add(PidIssuanceStatus.idle);
  }

  @override
  Stream<PidIssuanceStatus> observePidIssuanceStatus() => _digidAuthStatusController.stream;
}
