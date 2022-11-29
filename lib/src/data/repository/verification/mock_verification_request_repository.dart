import '../../../domain/model/data_attribute.dart';
import '../../../feature/verification/model/verification_request.dart';
import '../../../feature/verification/model/verifier_policy.dart';
import '../../source/organization_datasource.dart';
import 'verification_request_repository.dart';

class MockVerificationRequestRepository implements VerificationRequestRepository {
  final OrganizationDataSource organizationDataSource;

  MockVerificationRequestRepository(this.organizationDataSource);

  @override
  Future<VerificationRequest> getRequest(String sessionId) async {
    switch (sessionId) {
      case 'LOTTERY':
        return VerificationRequest(
          id: 'LOTTERY',
          organization: (await organizationDataSource.read('staatsloterij'))!,
          attributes: const [
            DataAttribute(type: DataAttributeType.text, label: 'Naam', value: 'John Doe'),
            DataAttribute(type: DataAttributeType.text, label: 'Email', value: 'john.doe@example.org'),
            DataAttribute(type: DataAttributeType.text, label: 'Telefoon', value: '+31623456789'),
            DataAttribute(type: DataAttributeType.text, label: 'Email', value: 'john.doe@example.org'),
          ],
          policy: _kMockLotteryPolicy,
        );
      case 'JOB_APPLICATION_INCOMPLETE':
        return VerificationRequest(
          id: 'JOB_APPLICATION_INCOMPLETE',
          organization: (await organizationDataSource.read('employer_1'))!,
          attributes: const [
            DataAttribute(type: DataAttributeType.text, label: 'Onderwijsinstelling', value: null),
            DataAttribute(type: DataAttributeType.text, label: 'Opleiding', value: null),
            DataAttribute(type: DataAttributeType.text, label: 'Type', value: null),
            DataAttribute(type: DataAttributeType.text, label: 'Uitgiftedatum', value: null),
          ],
          policy: _kEmployerPolicy,
        );
      case 'JOB_APPLICATION_COMPLETE':
        return VerificationRequest(
          id: 'JOB_APPLICATION_COMPLETE',
          organization: (await organizationDataSource.read('employer_1'))!,
          attributes: const [
            DataAttribute(type: DataAttributeType.text, label: 'Onderwijsinstelling', value: 'Universiteit X'),
            DataAttribute(type: DataAttributeType.text, label: 'Opleiding', value: 'WO Master Bedrijfskunde'),
            DataAttribute(type: DataAttributeType.text, label: 'Type', value: 'Getuigschrift'),
            DataAttribute(type: DataAttributeType.text, label: 'Uitgiftedatum', value: '1 januari 2013'),
          ],
          policy: _kEmployerPolicy,
        );
    }
    throw UnimplementedError('No mock usecase for id: $sessionId');
  }
}

const _kEmployerPolicy = VerifierPolicy(
  storageDuration: Duration(days: 3 * 30),
  dataPurpose: 'Gegevens controle',
  privacyPolicyUrl: 'https://www.example.org',
  deletionCanBeRequested: true,
  dataIsShared: false,
);

const _kMockLotteryPolicy = VerifierPolicy(
  storageDuration: Duration(days: 30),
  dataPurpose: 'Authenticatie',
  privacyPolicyUrl: 'https://www.example.org',
  deletionCanBeRequested: false,
  dataIsShared: true,
);
