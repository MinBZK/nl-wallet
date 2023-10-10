import '../../wallet/setup_mocked_wallet_usecase.dart';
import '../perform_pre_navigation_actions_usecase.dart';

class PerformPreNavigationActionsUseCaseImpl implements PerformPreNavigationActionsUseCase {
  final SetupMockedWalletUseCase _setupMockedWalletUseCase;

  PerformPreNavigationActionsUseCaseImpl(this._setupMockedWalletUseCase);

  @override
  Future<void> invoke(List<PreNavigationAction> actions) async {
    for (final action in actions) {
      switch (action) {
        case PreNavigationAction.setupMockedWallet:
          await _setupMockedWalletUseCase.invoke();
          break;
      }
    }
  }
}
