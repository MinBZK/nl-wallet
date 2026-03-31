import '../../../../data/repository/close_proximity/close_proximity_repository.dart';
import '../../../model/result/result.dart';
import '../start_close_proximity_disclosure_usecase.dart';

class StartCloseProximityDisclosureUseCaseImpl extends StartCloseProximityDisclosureUseCase {
  final CloseProximityRepository _closeProximityRepository;

  StartCloseProximityDisclosureUseCaseImpl(this._closeProximityRepository);

  @override
  Future<Result<String>> invoke() async {
    return tryCatch(
      _closeProximityRepository.startCloseProximityDisclosure,
      'failed to start close proximity disclosure',
    );
  }
}
