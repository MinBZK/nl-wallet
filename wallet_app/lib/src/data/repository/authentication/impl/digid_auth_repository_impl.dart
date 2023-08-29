import 'dart:async';

import 'package:rxdart/rxdart.dart';

import '../../../../../bridge_generated.dart';
import '../../../../wallet_core/typed_wallet_core.dart';
import '../digid_auth_repository.dart';

class DigidAuthRepositoryImpl extends DigidAuthRepository {
  final StreamController<DigidAuthStatus> _digidAuthStatusController = BehaviorSubject();
  final TypedWalletCore _walletCore;

  DigidAuthRepositoryImpl(this._walletCore);

  @override
  Future<String> getAuthUrl() => _walletCore.createPidIssuanceRedirectUri();

  @override
  void notifyDigidStateUpdate(DigidState? state) {
    switch (state) {
      case DigidState.Authenticating:
        _digidAuthStatusController.add(DigidAuthStatus.authenticating);
        break;
      case DigidState.Success:
        _digidAuthStatusController.add(DigidAuthStatus.success);
        _digidAuthStatusController.add(DigidAuthStatus.idle);
        break;
      case DigidState.Error:
        _digidAuthStatusController.add(DigidAuthStatus.error);
        _digidAuthStatusController.add(DigidAuthStatus.idle);
        break;
      case null:
        _digidAuthStatusController.add(DigidAuthStatus.idle);
        break;
    }
  }

  @override
  Stream<DigidAuthStatus> observeAuthStatus() => _digidAuthStatusController.stream;
}
