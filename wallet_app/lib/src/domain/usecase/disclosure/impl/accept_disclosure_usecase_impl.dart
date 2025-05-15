import '../../../model/result/result.dart';
import '../accept_disclosure_usecase.dart';

class AcceptDisclosureUseCaseImpl extends AcceptDisclosureUseCase {
  final DisclosureRepository _disclosureRepository;

  AcceptDisclosureUseCaseImpl(this._disclosureRepository);

  @override
  Future<Result<String?>> invoke(String pin) async {
    return tryCatch(
      () async => _disclosureRepository.acceptDisclosure(pin),
      'Failed to accept disclosure',
    );
  }
}
