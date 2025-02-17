import 'package:fimber/fimber.dart';

import '../../../../util/extension/core_error_extension.dart';
import '../../../../wallet_core/error/core_error.dart';
import '../../../model/result/application_error.dart';
import '../../../model/result/result.dart';
import '../cancel_disclosure_usecase.dart';

class CancelDisclosureUseCaseImpl extends CancelDisclosureUseCase {
  final DisclosureRepository _disclosureRepository;

  CancelDisclosureUseCaseImpl(this._disclosureRepository);

  @override
  Future<Result<String?>> invoke() async {
    try {
      final hasActiveSession = await _disclosureRepository.hasActiveDisclosureSession();
      if (hasActiveSession) {
        final result = await _disclosureRepository.cancelDisclosure();
        return Result.success(result);
      } else {
        Fimber.d('No active disclosure session to cancel');
        return const Result.success(null);
      }
    } on CoreError catch (ex) {
      final returnUrl = ex.returnUrl;
      // This matches the behaviour we previously had in disclosure_bloc, i.e. if the error still gives us a
      // returnUrl, we consider the session cancelled and redirect the user. Worth investigating if this behaviour
      // can realistically occur.
      if (returnUrl != null) return Result.success(returnUrl);
      return Result.error(await ex.asApplicationError());
    } catch (ex) {
      Fimber.e('Failed to cancel active disclosure session', ex: ex);
      return Result.error(GenericError(ex.toString(), sourceError: ex));
    }
  }
}
