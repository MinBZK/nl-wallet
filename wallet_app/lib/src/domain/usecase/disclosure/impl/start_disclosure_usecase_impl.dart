import '../start_disclosure_usecase.dart';

class StartDisclosureUseCaseImpl extends StartDisclosureUseCase {
  final DisclosureRepository _disclosureRepository;

  StartDisclosureUseCaseImpl(this._disclosureRepository);

  @override
  Stream<StartDisclosureResult> invoke(Uri disclosureUri) => _disclosureRepository.startDisclosure(disclosureUri);
}
