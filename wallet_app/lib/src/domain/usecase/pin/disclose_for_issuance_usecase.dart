import '../../model/card/wallet_card.dart';
import '../issuance/accept_issuance_usecase.dart';
import 'check_pin_usecase.dart';

export 'check_pin_usecase.dart';

/// Accept a 'disclosure based issuance' request, returns the card
/// previews that are available for issuance based on this request.
/// To add the cards to the user's wallet continue with [AcceptIssuanceUseCase].
abstract class DiscloseForIssuanceUseCase extends CheckPinUseCase {
  @override
  Future<Result<List<WalletCard>>> invoke(String pin);
}
