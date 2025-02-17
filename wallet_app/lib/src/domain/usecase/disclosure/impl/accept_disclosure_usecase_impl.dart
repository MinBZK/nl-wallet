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
        return result.map(
          ok: (ok) => ok.returnUrl,
          instructionError: (error) => throw IncorrectPinError(
            error.error.asCheckPinResult(),
            sourceError: error,
          ),
        );
      },
      'Failed to accept disclosure',
    );
  }
}
