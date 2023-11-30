import '../../model/navigation/navigation_request.dart';
import '../../model/wallet_card.dart';

abstract class GetWalletCardUpdateRequestUseCase {
  Future<NavigationRequest?> invoke(WalletCard card);
}
