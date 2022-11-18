import '../../../domain/model/card_front.dart';
import '../../../domain/model/data_attribute.dart';
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
    final organization = (await organizationDataSource.read('rvig'))!;
    switch (issuanceRequestId) {
      case 'PID_1':
        return IssuanceResponse(
          organization: organization,
          cards: [_kMockPidWalletCard],
          requestedAttributes: [],
        );
      case '1':
        return IssuanceResponse(
          organization: organization,
          cards: [_kMockPassportWalletCard],
          requestedAttributes: _kMockRequestedAttributes,
        );
      case '2':
        return IssuanceResponse(
          organization: organization,
          cards: [_kMockLicenseWalletCard],
          requestedAttributes: _kMockRequestedAttributes,
        );
    }
    throw UnsupportedError('Unknown issuer: $issuanceRequestId');
  }
}
