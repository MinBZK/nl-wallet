import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/attribute/requested_attribute.dart';
import '../../../../domain/model/card_front.dart';
import '../../../../domain/model/issuance_response.dart';
import '../../../../domain/model/policy/policy.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../source/mock/mock_organization_datasource.dart';
import '../../../source/organization_datasource.dart';
import '../../../source/wallet_datasource.dart';
import '../issuance_response_repository.dart';

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
          organization: (await organizationDataSource.read(_kMockPidWalletCard.issuerId))!,
          requestedAttributes: [],
          policy: _kMockIssuancePolicy,
          cards: [_kMockPidWalletCard, _kMockAddressWalletCard],
        );
      case _kDiplomaId:
        return IssuanceResponse(
          organization: (await organizationDataSource.read(_kMockDiplomaWalletCard.issuerId))!,
          requestedAttributes: _kMockGovernmentOrganizationRequestedAttributes,
          policy: _kMockIssuancePolicy,
          cards: [_kMockDiplomaWalletCard],
        );
      case _kMultiDiplomaId:
        return IssuanceResponse(
          organization: (await organizationDataSource.read(_kMockDiplomaWalletCard.issuerId))!,
          requestedAttributes: _kMockGovernmentOrganizationRequestedAttributes,
          policy: _kMockIssuancePolicy,
          cards: [_kMockDiplomaWalletCard, _kMockMasterDiplomaWalletCard],
        );
      case _kDrivingLicenseId:
        return IssuanceResponse(
          organization: (await organizationDataSource.read(_kMockDrivingLicenseWalletCard.issuerId))!,
          requestedAttributes: _kMockGovernmentOrganizationRequestedAttributes,
          policy: _kMockIssuancePolicy,
          cards: [_kMockDrivingLicenseWalletCard],
        );
      case _kDrivingLicenseRenewedId:
        return IssuanceResponse(
          organization: (await organizationDataSource.read(_kMockDrivingLicenseRenewedWalletCard.issuerId))!,
          requestedAttributes: _kMockGovernmentOrganizationRequestedAttributes,
          policy: _kMockIssuancePolicy,
          cards: [_kMockDrivingLicenseRenewedWalletCard],
        );
      case _kHealthInsuranceId:
        return IssuanceResponse(
          organization: (await organizationDataSource.read(_kMockHealthInsuranceWalletCard.issuerId))!,
          requestedAttributes: _kMockHealthInsuranceRequestedAttributes,
          policy: _kMockIssuancePolicy,
          cards: [_kMockHealthInsuranceWalletCard],
        );
      case _kVOGId:
        return IssuanceResponse(
          organization: (await organizationDataSource.read(_kMockVOGWalletCard.issuerId))!,
          requestedAttributes: _kMockGovernmentOrganizationRequestedAttributes,
          policy: _kMockIssuancePolicy,
          cards: [_kMockVOGWalletCard],
        );
    }
    throw UnsupportedError('Unknown issuer: $issuanceRequestId');
  }
}
