import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:rxdart/rxdart.dart';

import '../../../../data/repository/pid/pid_repository.dart';
import '../../../../util/cast_util.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../continue_pid_issuance_usecase.dart';

class ContinuePidIssuanceUseCaseImpl implements ContinuePidIssuanceUseCase {
  final PidRepository _pidRepository;

  ContinuePidIssuanceUseCaseImpl(this._pidRepository);

  @override
  Stream<PidIssuanceStatus> invoke(Uri uri) {
    return _pidRepository
        .continuePidIssuance(uri)
        .map<PidIssuanceStatus>((attributes) => PidIssuanceSuccess(attributes))
        .startWith(PidIssuanceAuthenticating())
        .onErrorReturnWith(
      (ex, stack) {
        Fimber.e('Pid issuance failed', ex: ex);
        return PidIssuanceError(_extractRedirectError(ex));
      },
    );
  }

  RedirectError _extractRedirectError(Object exception) {
    try {
      final redirectUriError = tryCast<CoreRedirectUriError>(exception);
      return redirectUriError?.redirectError ?? RedirectError.unknown;
    } catch (ex) {
      Fimber.e('Failed to extract RedirectError', ex: ex);
      return RedirectError.unknown;
    }
  }
}
