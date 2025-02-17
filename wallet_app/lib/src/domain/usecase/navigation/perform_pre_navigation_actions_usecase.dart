import '../../model/navigation/navigation_request.dart';
import '../wallet_usecase.dart';

export '../../model/navigation/navigation_request.dart';

abstract class PerformPreNavigationActionsUseCase extends WalletUseCase {
  /// Performs all the specified pre navigation actions
  Future<void> invoke(List<PreNavigationAction> actions);
}
