import '../../model/issuance/disclose_for_issuance_result.dart';
import '../issuance/accept_issuance_usecase.dart';
import '../issuance/commit_issued_cards_usecase.dart';
import 'check_pin_usecase.dart';

export 'check_pin_usecase.dart';

/// Accept a 'disclosure based issuance' request, returns the card previews that are available for issuance based on
/// this request. To add the cards to the user's wallet continue with [AcceptIssuanceUseCase].
///
/// When the issuer does not provide previews the cards are issued right away and returned for review,
/// they are added to the wallet with [CommitIssuedCardsUseCase].
abstract class DiscloseForIssuanceUseCase extends CheckPinUseCase {
  @override
  Future<Result<DiscloseForIssuanceResult>> invoke(String pin);
}
