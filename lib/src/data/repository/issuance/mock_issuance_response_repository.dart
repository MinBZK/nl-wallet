import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/attribute/requested_attribute.dart';
import '../../../domain/model/card_front.dart';
import '../../../domain/model/issuance_response.dart';
import '../../../domain/model/wallet_card.dart';
import '../../source/organization_datasource.dart';
import '../../source/wallet_datasource.dart';
import 'issuance_response_repository.dart';

part 'mock_issuance_response_repository.mocks.dart';

class MockIssuanceResponseRepository extends IssuanceResponseRepository {
  final WalletDataSource walletDataSource;
  final OrganizationDataSource organizationDataSource;

  MockIssuanceResponseRepository(this.walletDataSource, this.organizationDataSource);

  @override
  Future<IssuanceResponse> read(String issuanceRequestId) async {
    switch (issuanceRequestId) {
      case _kPidId:
        return IssuanceResponse(
          organization: (await organizationDataSource.read('rvig'))!,
          cards: [_kMockPidWalletCard],
          requestedAttributes: [],
        );
      case _kDiplomaId:
        return IssuanceResponse(
          organization: (await organizationDataSource.read('duo'))!,
          cards: [_kMockDiplomaWalletCard],
          requestedAttributes: _kMockDiplomaRequestedAttributes,
        );
      case _kDrivingLicenseId:
        return IssuanceResponse(
          organization: (await organizationDataSource.read('rdw'))!,
          cards: [_kMockDrivingLicenseWalletCard],
          requestedAttributes: _kMockGenericRequestedAttributes,
        );
      case _kHealthInsuranceId:
        return IssuanceResponse(
          organization: (await organizationDataSource.read('health_insurer_1'))!,
          cards: [_kMockHealthInsuranceWalletCard],
          requestedAttributes: _kMockHealthInsuranceRequestedAttributes,
        );
      case _kVOGId:
        return IssuanceResponse(
          organization: (await organizationDataSource.read('justis'))!,
          cards: [_kMockVOGWalletCard],
          requestedAttributes: _kMockGenericRequestedAttributes,
        );
    }
    throw UnsupportedError('Unknown issuer: $issuanceRequestId');
  }
}
