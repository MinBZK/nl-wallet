import 'package:wallet_core/core.dart';

import '../../../../util/extension/wallet_instruction_error_extension.dart';
import '../../../model/result/application_error.dart';
import '../../../model/result/result.dart';
import '../accept_disclosure_usecase.dart';

class AcceptDisclosureUseCaseImpl extends AcceptDisclosureUseCase {
  final DisclosureRepository _disclosureRepository;

  AcceptDisclosureUseCaseImpl(this._disclosureRepository);

  @override
  Future<Result<String?>> invoke(String pin) async {
    return tryCatch(
      () async {
        final result = await _disclosureRepository.acceptDisclosure(pin);
        return switch (result) {
          AcceptDisclosureResult_Ok() => result.returnUrl,
          AcceptDisclosureResult_InstructionError() => throw IncorrectPinError(
              result.error.asCheckPinResult(),
              sourceError: result,
            ),
        };
      },
      'Failed to accept disclosure',
    );
  }
}
