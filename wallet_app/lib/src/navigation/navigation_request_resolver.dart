import 'package:wallet_core/core.dart';

import '../domain/model/navigation/navigation_request.dart';
import '../feature/disclosure/argument/disclosure_screen_argument.dart';
import '../feature/issuance/argument/issuance_screen_argument.dart';

/// Maps an [IdentifyUriResult] (as returned by wallet_core) to a [NavigationRequest]
/// that can be used to trigger navigation using [NavigationService].
NavigationRequest resolveNavigationRequest(IdentifyUriResult uriType, String uri, {required bool isQrCode}) {
  switch (uriType) {
    case IdentifyUriResult.PidIssuance:
      return NavigationRequest.pidIssuance(uri);
    case IdentifyUriResult.PidRenewal:
      return NavigationRequest.pidRenewal(uri);
    case IdentifyUriResult.PinRecovery:
      return NavigationRequest.pinRecovery(uri);
    case IdentifyUriResult.Disclosure:
      return NavigationRequest.disclosure(
        argument: DisclosureScreenArgument(type: .remote(uri, isQrCode: isQrCode)),
      );
    case IdentifyUriResult.DisclosureBasedIssuance:
      return NavigationRequest.issuance(
        argument: IssuanceScreenArgument(uri: uri, isQrCode: isQrCode, issuanceType: .disclosureBasedIssuance),
      );
    case IdentifyUriResult.Transfer:
      return NavigationRequest.walletTransferSource(uri);
    case IdentifyUriResult.CredentialOffer:
      return NavigationRequest.issuance(
        argument: IssuanceScreenArgument(uri: uri, isQrCode: isQrCode, issuanceType: .credentialOffer),
      );
    case IdentifyUriResult.GenericIssuance:
      return NavigationRequest.continueIssuance(
        argument: IssuanceScreenArgument(uri: uri, isQrCode: isQrCode, issuanceType: .authorizationCallback),
      );
  }
}
