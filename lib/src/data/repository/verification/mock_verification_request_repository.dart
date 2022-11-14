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
      case '1':
        return VerificationRequest(
          id: '1',
          organization: (await organizationDataSource.read('duo'))!,
          attributes: const [
            DataAttribute(type: 'Niveau', value: 'Master - WO'),
            DataAttribute(type: 'Onderwijsinstelling', value: 'Technische Universiteit Delft'),
            DataAttribute(type: 'Opleidingsnaam', value: 'Integrated Product Design'),
            DataAttribute(type: 'Verklaring Omtrent het Gedrag', value: 'Profiel 11, 12, 13'),
          ],
          policy: _kMockDuoPolicy,
        );
      case '2':
        return VerificationRequest(
          id: '2',
          organization: (await organizationDataSource.read('staatsloterij'))!,
          attributes: const [
            DataAttribute(type: 'Naam', value: 'John Doe'),
            DataAttribute(type: 'Email', value: 'john.doe@example.org'),
            DataAttribute(type: 'Telefoon', value: '+31623456789'),
            DataAttribute(type: 'Email', value: 'john.doe@example.org'),
          ],
          policy: _kMockLotteryPolicy,
        );
      case '3':
        return VerificationRequest(
          id: '3',
          organization: (await organizationDataSource.read('duo'))!,
          attributes: const [
            DataAttribute(type: 'Onderwijsinstelling', value: null),
            DataAttribute(type: 'Verklaring Omtrent het Gedrag', value: null),
          ],
          policy: _kMockDuoPolicy,
        );
    }
    throw UnimplementedError('No mock usecase for id: $sessionId');
  }
}

const _kMockDuoPolicy = VerifierPolicy(
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
