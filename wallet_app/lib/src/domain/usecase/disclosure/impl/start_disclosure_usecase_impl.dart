import '../../../model/disclosure/start_disclosure_request.dart';
import '../../../model/result/result.dart';
import '../start_disclosure_usecase.dart';

class StartDisclosureUseCaseImpl extends StartDisclosureUseCase {
  final DisclosureRepository _disclosureRepository;

  StartDisclosureUseCaseImpl(this._disclosureRepository);

  @override
  Future<Result<StartDisclosureResult>> invoke(StartDisclosureRequest request) async {
    return tryCatch(() async {
      switch (request) {
        case DeeplinkStartDisclosureRequest(:final uri):
          return _disclosureRepository.startDisclosure(uri, isQrCode: false);
        case QrScanStartDisclosureRequest(:final uri):
          return _disclosureRepository.startDisclosure(uri, isQrCode: true);
        case CloseProximityStartDisclosureRequest():
          return _disclosureRepository.continueCloseProximityDisclosure();
      }
    }, 'Failed to start disclosure');
  }
}
