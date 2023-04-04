import '../../../../data/repository/issuance/issuance_response_repository.dart';
import '../../../model/issuance_response.dart';
import '../get_my_government_issuance_responses_usecase.dart';

const _kDiplomaCardId = 'DIPLOMA_1';
const _kDrivingLicenseCardId = 'DRIVING_LICENSE';

class GetMyGovernmentIssuanceResponsesUseCaseImpl implements GetMyGovernmentIssuanceResponsesUseCase {
  final IssuanceResponseRepository issuanceResponseRepository;

  const GetMyGovernmentIssuanceResponsesUseCaseImpl(this.issuanceResponseRepository);

  @override
  Future<List<IssuanceResponse>> invoke() async {
    final drivingLicense = await issuanceResponseRepository.read(_kDrivingLicenseCardId);
    final diploma = await issuanceResponseRepository.read(_kDiplomaCardId);
    return [drivingLicense, diploma];
  }
}
