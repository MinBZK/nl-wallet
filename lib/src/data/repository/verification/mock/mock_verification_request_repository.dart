import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/attribute/requested_attribute.dart';
import '../../../../domain/model/policy/interaction_policy.dart';
import '../../../../feature/verification/model/verification_request.dart';
import '../../../source/organization_datasource.dart';
import '../verification_request_repository.dart';

part 'mock_verification_request_repository.mocks.dart';

class MockVerificationRequestRepository implements VerificationRequestRepository {
  final OrganizationDataSource organizationDataSource;

  MockVerificationRequestRepository(this.organizationDataSource);

  @override
  Future<VerificationRequest> getRequest(String sessionId) async {
    switch (sessionId) {
      case _kJobApplicationId:
        return VerificationRequest(
          id: _kJobApplicationId,
          organization: (await organizationDataSource.read('employer_1'))!,
          requestedAttributes: _kJobApplicationRequestedAttributes,
          interactionPolicy: _kEmployerPolicy,
        );
      case _kMarketplaceLoginId:
        return VerificationRequest(
          id: _kMarketplaceLoginId,
          organization: (await organizationDataSource.read('marketplace'))!,
          requestedAttributes: _kMarketplaceLoginRequestedAttributes,
          interactionPolicy: _kMockMarketPlacePolicy,
        );
      case _kBarId:
        return VerificationRequest(
          id: _kBarId,
          organization: (await organizationDataSource.read('bar'))!,
          requestedAttributes: _kBarRequestedAttributes,
          interactionPolicy: _kMockBarPolicy,
        );
      case _kCarRental:
        return VerificationRequest(
          id: _kCarRental,
          organization: (await organizationDataSource.read('car_rental'))!,
          requestedAttributes: _kCarRentalRequestedAttributes,
          interactionPolicy: _kMockCarRentalPolicy,
        );
      case _kFirstAid:
        return VerificationRequest(
          id: _kFirstAid,
          organization: (await organizationDataSource.read('first_aid'))!,
          requestedAttributes: _kFirstAidRequestedAttributes,
          interactionPolicy: _kMockCarRentalPolicy,
        );
    }
    throw UnimplementedError('No mock usecase for id: $sessionId');
  }
}
