import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../model/navigation/navigation_request.dart';
import '../../../model/wallet_state.dart';
import '../check_navigation_prerequisites_usecase.dart';

class CheckNavigationPrerequisitesUseCaseImpl extends CheckNavigationPrerequisitesUseCase {
  final WalletRepository _walletRepository;

  CheckNavigationPrerequisitesUseCaseImpl(this._walletRepository);

  @override
  Future<bool> invoke(List<NavigationPrerequisite> prerequisites) async {
    for (final prerequisite in prerequisites) {
      final isMet = await _isPrerequisiteMet(prerequisite);
      if (!isMet) return false;
    }
    return true;
  }

  Future<bool> _isPrerequisiteMet(NavigationPrerequisite prerequisite) async {
    switch (prerequisite) {
      case NavigationPrerequisite.walletUnlocked:
        return !(await _walletRepository.isLockedStream.first);
      case NavigationPrerequisite.walletInitialized:
        return _walletRepository.isRegistered();
      case NavigationPrerequisite.pidInitialized:
        return _walletRepository.containsPid();
      case NavigationPrerequisite.walletInReadyState:
        return (await _walletRepository.getWalletState()) is WalletStateReady;
      case NavigationPrerequisite.walletInIssuanceState:
        return (await _walletRepository.getWalletState()) is WalletStateInIssuanceFlow;
    }
  }
}
