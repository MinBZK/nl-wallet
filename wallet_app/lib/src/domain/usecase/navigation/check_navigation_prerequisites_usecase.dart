import '../../model/navigation/navigation_request.dart';
import '../wallet_usecase.dart';

abstract class CheckNavigationPrerequisitesUseCase extends WalletUseCase {
  /// Returns true when all navigationPrerequisites are passed
  Future<bool> invoke(List<NavigationPrerequisite> prerequisites);
}
