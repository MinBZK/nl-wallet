import 'dart:async';

import 'package:fimber/fimber.dart';

import '../../../../data/repository/pid/pid_repository.dart';
import '../../../../util/cast_util.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../continue_pid_issuance_usecase.dart';

class ContinuePidIssuanceUseCaseImpl implements ContinuePidIssuanceUseCase {
  final PidRepository _pidRepository;

  ContinuePidIssuanceUseCaseImpl(this._pidRepository);

  @override
  Future<PidIssuanceStatus> invoke(String uri) async {
    try {
      final result = await _pidRepository.continuePidIssuance(uri);
      return PidIssuanceSuccess(result);
    } catch (ex) {
      Fimber.e('Failed to continue pid issuance', ex: ex);
      return PidIssuanceError(_extractRedirectError(ex));
    }
  }

  /// Try to extract a [RedirectError] from the provided exception. Throws the
  /// original exception if the conversion fails.
  RedirectError _extractRedirectError(Object exception) {
    final redirectUriError = tryCast<CoreRedirectUriError>(exception);
    if (redirectUriError == null) throw exception;
    return redirectUriError.redirectError;
  }
}
