import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:rxdart/rxdart.dart';

import '../../../../../bridge_generated.dart';
import '../../../../util/cast_util.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../../../../wallet_core/error/core_error_mapper.dart';
import '../../../../wallet_core/typed_wallet_core.dart';
import '../pid_repository.dart';

class PidRepositoryImpl extends PidRepository {
  final StreamController<PidIssuanceStatus> _digidAuthStatusController = BehaviorSubject();
  final TypedWalletCore _walletCore;
  final CoreErrorMapper _errorMapper;

  PidRepositoryImpl(this._walletCore, this._errorMapper);

  @override
  Future<String> getPidIssuanceUrl() => _walletCore.createPidIssuanceRedirectUri();

  @override
  void notifyPidIssuanceStateUpdate(PidIssuanceEvent? event) {
    event?.when(
      authenticating: () {
        _digidAuthStatusController.add(PidIssuanceAuthenticating());
      },
      success: (success) {
        //TODO: Pass on cards
        _digidAuthStatusController.add(PidIssuanceSuccess(List.empty()));
        _digidAuthStatusController.add(PidIssuanceIdle());
      },
      error: (error) {
        _digidAuthStatusController.add(PidIssuanceError(_extractRedirectError(error)));
        _digidAuthStatusController.add(PidIssuanceIdle());
      },
    );
    if (event == null) _digidAuthStatusController.add(PidIssuanceIdle());
  }

  RedirectError _extractRedirectError(String flutterApiErrorJson) {
    try {
      final coreError = _errorMapper.map(flutterApiErrorJson);
      final redirectUriError = tryCast<CoreRedirectUriError>(coreError);
      return redirectUriError?.redirectError ?? RedirectError.unknown;
    } catch (ex) {
      Fimber.e('Failed to extract RedirectError', ex: ex);
      return RedirectError.unknown;
    }
  }

  @override
  Stream<PidIssuanceStatus> observePidIssuanceStatus() => _digidAuthStatusController.stream;
}
