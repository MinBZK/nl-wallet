import '../../../domain/model/data_attribute.dart';
import '../../../domain/model/requested_attribute.dart';
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
          requestedAttributes: const [
            RequestedAttribute(
              name: 'Voornaam',
              type: DataAttributeType.firstName,
              valueType: DataAttributeValueType.text,
            ),
            RequestedAttribute(
              name: 'Achternaam',
              type: DataAttributeType.lastName,
              valueType: DataAttributeValueType.text,
            ),
            RequestedAttribute(
              name: 'BSN Nummer',
              type: DataAttributeType.citizenshipNumber,
              valueType: DataAttributeValueType.text,
            ),
          ],
          policy: _kMockLotteryPolicy,
        );
      case 'JOB_APPLICATION':
        return VerificationRequest(
          id: 'JOB_APPLICATION',
          organization: (await organizationDataSource.read('employer_1'))!,
          requestedAttributes: const [
            RequestedAttribute(
              name: 'Opleidingsnaam',
              type: DataAttributeType.education,
              valueType: DataAttributeValueType.text,
            ),
            RequestedAttribute(
                name: 'Onderwijsinstelling',
                type: DataAttributeType.university,
                valueType: DataAttributeValueType.text),
            RequestedAttribute(
              name: 'Niveau',
              type: DataAttributeType.educationLevel,
              valueType: DataAttributeValueType.text,
            ),
          ],
          policy: _kEmployerPolicy,
        );
      case 'BAR':
        return VerificationRequest(
          id: 'BAR',
          organization: (await organizationDataSource.read('bar'))!,
          requestedAttributes: const [
            RequestedAttribute(
              name: 'Pasfoto',
              type: DataAttributeType.profilePhoto,
              valueType: DataAttributeValueType.image,
            ),
            RequestedAttribute(
              name: 'Ouder dan 18',
              type: DataAttributeType.olderThan18,
              valueType: DataAttributeValueType.text,
            ),
          ],
          policy: _kMockBarPolicy,
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

const _kMockBarPolicy = VerifierPolicy(
  storageDuration: Duration(days: 0),
  dataPurpose: 'Leeftijd controle',
  privacyPolicyUrl: 'https://www.example.org',
  deletionCanBeRequested: true,
  dataIsShared: false,
);
