import '../../../data/repository/issuance/issuance_response_repository.dart';
import '../../model/issuance_response.dart';
import '../../model/wallet_card.dart';

const _kDrivingLicenseId = 'DRIVING_LICENSE';
const _kDrivingLicenseRenewedId = 'DRIVING_LICENSE_RENEWED';

class GetWalletCardUpdateIssuanceRequestIdUseCase {
  final IssuanceResponseRepository issuanceResponseRepository;

  GetWalletCardUpdateIssuanceRequestIdUseCase(this.issuanceResponseRepository);

  /// Returns an `Issuance Request ID` to be used within the Issuance flow;
  /// In case new data attributes are available, else `null` is returned.
  Future<String?> invoke(WalletCard card) async {
    switch (card.id) {
      case _kDrivingLicenseId:

        // Retrieve latest (mocked) card + attributes
        IssuanceResponse response = await issuanceResponseRepository.read(_kDrivingLicenseRenewedId);
        WalletCard cardLatestVersion = response.cards.first;

        // Compare current card attributes with latest card version/attributes
        if (card.attributes != cardLatestVersion.attributes) {
          return _kDrivingLicenseRenewedId;
        }
        break;
    }
    return null;
  }
}
