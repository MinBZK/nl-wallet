import '../cancel_disclosure_usecase.dart';

class CancelDisclosureUseCaseImpl extends CancelDisclosureUseCase {
  final DisclosureRepository _disclosureRepository;

  CancelDisclosureUseCaseImpl(this._disclosureRepository);

  @override
  Future<void> invoke() => _disclosureRepository.cancelDisclosure();
}
