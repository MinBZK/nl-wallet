import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../model/navigation/navigation_request.dart';
import '../check_navigation_prerequisites_usecase.dart';

class CheckNavigationPrerequisitesUseCaseImpl implements CheckNavigationPrerequisitesUseCase {
  final WalletRepository _walletRepository;

  CheckNavigationPrerequisitesUseCaseImpl(this._walletRepository);

  @override
  Future<bool> invoke(List<NavigationPrerequisite> prerequisites) async {
    for (final prerequisite in prerequisites) {
      switch (prerequisite) {
        case NavigationPrerequisite.walletUnlocked:
          final isLocked = await _walletRepository.isLockedStream.first;
          if (isLocked) return false;
          break;
        case NavigationPrerequisite.walletInitialized:
          final isInitialized = await _walletRepository.isRegistered();
          if (!isInitialized) return false;
          break;
        case NavigationPrerequisite.pidInitialized:
          final containsPid = await _walletRepository.containsPid();
          if (!containsPid) return false;
          break;
      }
    }
    return true;
  }
}
