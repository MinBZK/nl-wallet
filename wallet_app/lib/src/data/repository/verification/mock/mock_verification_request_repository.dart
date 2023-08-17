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
          requestPurpose: 'Sollicitatie',
          interactionPolicy: _kEmployerPolicy,
        );
      case _kMarketplaceLoginId:
        return VerificationRequest(
          id: _kMarketplaceLoginId,
          organization: (await organizationDataSource.read(kMarketplaceId))!,
          requestedAttributes: _kMarketplaceLoginRequestedAttributes,
          requestPurpose: 'Account aanmaken',
          interactionPolicy: _kMockMarketPlacePolicy,
        );
      case _kBarId:
        return VerificationRequest(
          id: _kBarId,
          organization: (await organizationDataSource.read(kBarId))!,
          requestedAttributes: _kBarRequestedAttributes,
          requestPurpose: 'Leeftijd controle',
          interactionPolicy: _kMockBarPolicy,
        );
      case _kCarRental:
        return VerificationRequest(
          id: _kCarRental,
          organization: (await organizationDataSource.read(kCarRentalId))!,
          requestedAttributes: _kCarRentalRequestedAttributes,
          requestPurpose: 'Gegevens & Rijbewijs controle',
          interactionPolicy: _kMockCarRentalPolicy,
        );
      case _kFirstAid:
        return VerificationRequest(
          id: _kFirstAid,
          organization: (await organizationDataSource.read(kFirstAidId))!,
          requestedAttributes: _kFirstAidRequestedAttributes,
          requestPurpose: 'Gegevens & Verzekering controle',
          interactionPolicy: _kMockFirstAidPolicy,
        );
      case _kParkingPermit:
        return VerificationRequest(
          id: _kParkingPermit,
          organization: (await organizationDataSource.read(kMunicipalityTheHagueId))!,
          requestedAttributes: _kParkingPermitRequestedAttributes,
          requestPurpose: 'Parkeervergunning',
          interactionPolicy: _kMockMunicipalityGenericPolicy,
        );
      case _kOpenBankAccount:
        return VerificationRequest(
          id: _kOpenBankAccount,
          organization: (await organizationDataSource.read(kBankId))!,
          requestedAttributes: _kOpenBankAccountRequestedAttributes,
          requestPurpose: 'Rekening openen',
          interactionPolicy: _kMockBankPolicy,
        );
      case _kProvideContractDetails:
        return VerificationRequest(
          id: _kProvideContractDetails,
          organization: (await organizationDataSource.read(kHousingCorpId))!,
          requestedAttributes: _kProvideContractDetailsRequestedAttributes,
          requestPurpose: 'Identificeren',
          interactionPolicy: _kMockHousingCorpPolicy,
        );
      case _kCreateMonkeyBikeAccount:
        return VerificationRequest(
          id: _kCreateMonkeyBikeAccount,
          organization: (await organizationDataSource.read(kMonkeyBikeId))!,
          requestedAttributes: _kCreateMbAccountRequestedAttributes,
          requestPurpose: 'Account aanmaken',
          interactionPolicy: _kMonkeyBikePolicy,
        );
    }
    throw UnimplementedError('No mock usecase for id: $requestId');
  }
}
