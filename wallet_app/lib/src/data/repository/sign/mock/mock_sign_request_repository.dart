import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/attribute/requested_attribute.dart';
import '../../../../domain/model/document.dart';
import '../../../../domain/model/policy/policy.dart';
import '../../../../domain/model/sign_request.dart';
import '../../../../domain/model/trust_provider.dart';
import '../../../../wallet_assets.dart';
import '../../../source/organization_datasource.dart';
import '../sign_request_repository.dart';

class MockSignRequestRepository implements SignRequestRepository {
  final OrganizationDataSource organizationDataSource;

  MockSignRequestRepository(this.organizationDataSource);

  @override
  Future<SignRequest> getRequest(String sessionId) async {
    switch (sessionId) {
      case 'RENTAL_AGREEMENT':
        return SignRequest(
          id: 'RENTAL_AGREEMENT',
          organization: (await organizationDataSource.read('housing_corp_1'))!,
          trustProvider: const TrustProvider(
            name: 'Veilig Ondertekenen B.V.',
            logoUrl: WalletAssets.logo_sign_provider,
          ),
          document: const Document(
            title: 'Huurovereenkomst',
            fileName: '230110_Huurcontract_Bruijn.pdf',
            url: 'path/to/sample.pdf',
          ),
          requestedAttributes: const [
            RequestedAttribute(
              label: 'Voornamen',
              key: 'mock.firstNames',
              valueType: AttributeValueType.text,
            ),
            RequestedAttribute(
              label: 'Achternaam',
              key: 'mock.lastName',
              valueType: AttributeValueType.text,
            ),
            RequestedAttribute(
              label: 'Geboortedatum',
              key: 'mock.birthDate',
              valueType: AttributeValueType.text,
            ),
          ],
          policy: _kMockSignPolicy,
        );
    }
    throw UnimplementedError('No mock usecase for id: $sessionId');
  }
}

const _kMockSignPolicy = Policy(
  storageDuration: null,
  dataPurpose: null,
  dataIsShared: false,
  dataIsSignature: true,
  dataContainsSingleViewProfilePhoto: false,
  deletionCanBeRequested: false,
  privacyPolicyUrl: null,
);
