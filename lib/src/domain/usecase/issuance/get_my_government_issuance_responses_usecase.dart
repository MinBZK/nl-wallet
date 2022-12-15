import '../../../data/repository/issuance/issuance_response_repository.dart';
import '../../model/issuance_response.dart';

const _kDiplomaCardId = 'DIPLOMA_1';
const _kDrivingLicenseCardId = 'DRIVING_LICENSE';

class GetMyGovernmentIssuanceResponsesUseCase {
  final IssuanceResponseRepository issuanceResponseRepository;

  const GetMyGovernmentIssuanceResponsesUseCase(this.issuanceResponseRepository);

  Future<List<IssuanceResponse>> invoke() async {
    final drivingLicense = await issuanceResponseRepository.read(_kDrivingLicenseCardId);
    final diploma = await issuanceResponseRepository.read(_kDiplomaCardId);
    return [drivingLicense, diploma];
  }
}
