import '../../../model/result/result.dart';
import '../start_disclosure_usecase.dart';

class StartDisclosureUseCaseImpl extends StartDisclosureUseCase {
  final DisclosureRepository _disclosureRepository;

  StartDisclosureUseCaseImpl(this._disclosureRepository);

  @override
  Future<Result<StartDisclosureResult>> invoke(String disclosureUri, {bool isQrCode = false}) async {
    return tryCatch(
      () async => _disclosureRepository.startDisclosure(disclosureUri, isQrCode: isQrCode),
      'Failed to start disclosure',
    );
  }
}
