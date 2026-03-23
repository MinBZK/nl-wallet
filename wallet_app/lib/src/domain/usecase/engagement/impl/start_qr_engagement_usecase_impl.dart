import '../../../../data/repository/close_proximity/close_proximity_repository.dart';
import '../../../model/result/result.dart';
import '../start_qr_engagement_usecase.dart';

class StartQrEngagementUseCaseImpl extends StartQrEngagementUseCase {
  final CloseProximityRepository _closeProximityRepository;

  StartQrEngagementUseCaseImpl(this._closeProximityRepository);

  @override
  Future<Result<String>> invoke() async {
    return tryCatch(
      _closeProximityRepository.startCloseProximityDisclosure,
      'failed to start ble server',
    );
  }
}
