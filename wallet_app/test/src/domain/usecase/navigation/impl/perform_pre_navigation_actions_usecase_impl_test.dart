import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/domain/usecase/navigation/impl/perform_pre_navigation_actions_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockSetupMockedWalletUseCase mockSetupMockedWalletUseCase;

  late PerformPreNavigationActionsUseCaseImpl usecase;

  setUp(() {
    mockSetupMockedWalletUseCase = MockSetupMockedWalletUseCase();
    usecase = PerformPreNavigationActionsUseCaseImpl(mockSetupMockedWalletUseCase);
  });

  group('invoke', () {
    test('When no pre navigation actions are requested, no extra usecase is invoked', () async {
      await usecase.invoke([]);
      verifyZeroInteractions(mockSetupMockedWalletUseCase);
    });

    test('When the action setupMockedWallet is requested, its usecase should be invoked', () async {
      await usecase.invoke([PreNavigationAction.setupMockedWallet]);
      verify(mockSetupMockedWalletUseCase.invoke()).called(1);
    });
  });
}
