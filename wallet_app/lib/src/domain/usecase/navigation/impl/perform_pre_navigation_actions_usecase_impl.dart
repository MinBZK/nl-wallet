import '../../../../navigation/secured_page_route.dart';
import '../../wallet/setup_mocked_wallet_usecase.dart';
import '../perform_pre_navigation_actions_usecase.dart';

class PerformPreNavigationActionsUseCaseImpl extends PerformPreNavigationActionsUseCase {
  final SetupMockedWalletUseCase _setupMockedWalletUseCase;

  PerformPreNavigationActionsUseCaseImpl(this._setupMockedWalletUseCase);

  @override
  Future<void> invoke(List<PreNavigationAction> actions) async {
    for (final action in actions) {
      switch (action) {
        case PreNavigationAction.setupMockedWallet:
          await _setupMockedWalletUseCase.invoke();
        case PreNavigationAction.disableUpcomingPageTransition:
          SecuredPageRoute.overrideDurationOfNextTransition(Duration.zero);
      }
    }
  }
}
