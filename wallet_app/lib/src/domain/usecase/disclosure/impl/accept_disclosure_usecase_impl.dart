import 'package:fimber/fimber.dart';

import '../../../../util/extension/accept_disclosure_result_extension.dart';
import '../../../model/pin/check_pin_result.dart';
import '../accept_disclosure_usecase.dart';

class AcceptDisclosureUseCaseImpl extends AcceptDisclosureUseCase {
  final DisclosureRepository _disclosureRepository;

  AcceptDisclosureUseCaseImpl(this._disclosureRepository);

  @override
  Future<CheckPinResult> invoke(String pin) async {
    try {
      final result = await _disclosureRepository.acceptDisclosure(pin);
      return result.asCheckPinResult();
    } catch (ex) {
      Fimber.e('Failed to accept disclosure', ex: ex);
      rethrow;
    }
  }
}
