import 'package:fimber/fimber.dart';

import '../cancel_disclosure_usecase.dart';

class CancelDisclosureUseCaseImpl extends CancelDisclosureUseCase {
  final DisclosureRepository _disclosureRepository;

  CancelDisclosureUseCaseImpl(this._disclosureRepository);

  @override
  Future<void> invoke() async {
    Fimber.d('Cancelling active disclosure session');
    final hasActiveSession = await _disclosureRepository.hasActiveDisclosureSession();
    if (hasActiveSession) {
      await _disclosureRepository.cancelDisclosure();
    } else {
      Fimber.e('No active disclosure session to cancel!');
    }
  }
}
