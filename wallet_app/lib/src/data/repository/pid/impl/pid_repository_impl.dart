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
  final StreamController<PidIssuanceStatus> _pidIssuanceStatusController = BehaviorSubject();
  final TypedWalletCore _walletCore;
  final CoreErrorMapper _errorMapper;

  PidRepositoryImpl(this._walletCore, this._errorMapper);

  @override
  Future<String> getPidIssuanceUrl() => _walletCore.createPidIssuanceRedirectUri();

  @override
  void notifyPidIssuanceStateUpdate(PidIssuanceEvent? event) {
    event?.when(
      authenticating: () {
        _pidIssuanceStatusController.add(PidIssuanceAuthenticating());
      },
      success: (success) {
        //TODO: Pass on cards
        _pidIssuanceStatusController.add(PidIssuanceSuccess(List.empty()));
        _pidIssuanceStatusController.add(PidIssuanceIdle());
      },
      error: (error) {
        _pidIssuanceStatusController.add(PidIssuanceError(_extractRedirectError(error)));
        _pidIssuanceStatusController.add(PidIssuanceIdle());
      },
    );
    if (event == null) _pidIssuanceStatusController.add(PidIssuanceIdle());
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
  Stream<PidIssuanceStatus> observePidIssuanceStatus() => _pidIssuanceStatusController.stream;
}
