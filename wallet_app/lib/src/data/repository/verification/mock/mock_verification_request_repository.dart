import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/attribute/requested_attribute.dart';
import '../../../../domain/model/policy/policy.dart';
import '../../../../feature/verification/model/verification_request.dart';
import '../../../source/mock/mock_organization_datasource.dart';
import '../../../source/organization_datasource.dart';
import '../verification_request_repository.dart';

part 'mock_verification_request_repository.mocks.dart';

class MockVerificationRequestRepository implements VerificationRequestRepository {
  final OrganizationDataSource organizationDataSource;

  MockVerificationRequestRepository(this.organizationDataSource);

  @override
  Future<VerificationRequest> getRequest(String requestId) async {
    switch (requestId) {
      case _kJobApplicationId:
        return VerificationRequest(
          id: _kJobApplicationId,
          organization: (await organizationDataSource.read(kEmployerId))!,
          requestedAttributes: _kJobApplicationRequestedAttributes,
          interactionPolicy: _kEmployerPolicy,
          purpose: 'Sollicitatie',
        );
      case _kMarketplaceLoginId:
        return VerificationRequest(
          id: _kMarketplaceLoginId,
          organization: (await organizationDataSource.read(kMarketplaceId))!,
          requestedAttributes: _kMarketplaceLoginRequestedAttributes,
          interactionPolicy: _kMockMarketPlacePolicy,
          purpose: 'Account aanmaken',
        );
      case _kBarId:
        return VerificationRequest(
          id: _kBarId,
          organization: (await organizationDataSource.read(kBarId))!,
          requestedAttributes: _kBarRequestedAttributes,
          interactionPolicy: _kMockBarPolicy,
          purpose: 'Leeftijd controle',
        );
      case _kCarRental:
        return VerificationRequest(
          id: _kCarRental,
          organization: (await organizationDataSource.read(kCarRentalId))!,
          requestedAttributes: _kCarRentalRequestedAttributes,
          interactionPolicy: _kMockCarRentalPolicy,
          purpose: 'Gegevens & Rijbewijs controle',
        );
      case _kFirstAid:
        return VerificationRequest(
          id: _kFirstAid,
          organization: (await organizationDataSource.read(kFirstAidId))!,
          requestedAttributes: _kFirstAidRequestedAttributes,
          interactionPolicy: _kMockFirstAidPolicy,
          purpose: 'Gegevens & Verzekering controle',
        );
      case _kParkingPermit:
        return VerificationRequest(
          id: _kParkingPermit,
          organization: (await organizationDataSource.read(kMunicipalityDelftId))!,
          requestedAttributes: _kParkingPermitRequestedAttributes,
          interactionPolicy: _kMockMunicipalityDelftPolicy,
          purpose: 'Parkeervergunning',
        );
      case _kOpenBankAccount:
        return VerificationRequest(
          id: _kOpenBankAccount,
          organization: (await organizationDataSource.read(kBankId))!,
          requestedAttributes: _kOpenBankAccountRequestedAttributes,
          interactionPolicy: _kMockBankPolicy,
          purpose: 'Rekening openen',
        );
      case _kProvideContractDetails:
        return VerificationRequest(
          id: _kProvideContractDetails,
          organization: (await organizationDataSource.read(kHousingCorpId))!,
          requestedAttributes: _kProvideContractDetailsRequestedAttributes,
          interactionPolicy: _kMockHousingCorpPolicy,
          purpose: 'Identificeren',
        );
      case _kCreateMonkeyBikeAccount:
        return VerificationRequest(
          id: _kCreateMonkeyBikeAccount,
          organization: (await organizationDataSource.read(kMonkeyBikeId))!,
          requestedAttributes: _kCreateMbAccountRequestedAttributes,
          interactionPolicy: _kMonkeyBikePolicy,
          purpose: 'Account aanmaken',
        );
    }
    throw UnimplementedError('No mock usecase for id: $requestId');
  }
}
