import 'package:wallet_mock/mock.dart';

import '../../../model/attribute/attribute.dart';
import '../../../model/navigation/navigation_request.dart';
import '../../../model/wallet_card.dart';
import '../get_wallet_card_update_request_usecase.dart';

const _kRenewDrivingLicenseUri =
    'walletdebuginteraction://deeplink#%7B%22id%22%3A%22DRIVING_LICENSE_RENEWED%22%2C%22type%22%3A%22issue%22%7D';

//FIXME: This still relies on knowledge about the mock, but is also out of scope for the MVP.
class GetWalletCardUpdateRequestUseCaseImpl implements GetWalletCardUpdateRequestUseCase {
  GetWalletCardUpdateRequestUseCaseImpl();

  /// Returns an `NavigationRequest` to launch an issuance flow to renew the card, if this
  /// is possible. Otherwise simply returns null.
  @override
  Future<NavigationRequest?> invoke(WalletCard card) async {
    if (card.docType == kDrivingLicenseDocType) {
      // Check if we already have the renewed card by checking the title, this heavily relies on knowledge of the mock.
      final allStringAttributeValues = card.attributes.map((e) => e.value).whereType<StringValue>().map((e) => e.value);
      final isUpdatedCard = allStringAttributeValues.any((element) => element.contains('C1'));
      if (!isUpdatedCard) return IssuanceNavigationRequest(_kRenewDrivingLicenseUri, isRefreshFlow: true);
    }
    return null;
  }
}
