import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/feature/menu/bloc/menu_bloc.dart';

import '../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockLockWalletUseCase lockWalletUsecase;
  late MockNavigationService navigationService;

  setUp(() {
    lockWalletUsecase = MockLockWalletUseCase();
    navigationService = MockNavigationService();
  });

  blocTest(
    'verify initial state',
    build: () => MenuBloc(lockWalletUsecase, navigationService),
    verify: (bloc) => expect(bloc.state, const MenuInitial()),
  );

  blocTest(
    'MenuLockWalletPressed invokes the lock usecase and dashboard navigation request',
    build: () => MenuBloc(lockWalletUsecase, navigationService),
    act: (bloc) => bloc.add(MenuLockWalletPressed()),
    verify: (bloc) {
      verify(lockWalletUsecase.invoke()).called(1);
      verify(
        navigationService.handleNavigationRequest(
          NavigationRequest.dashboard(),
          queueIfNotReady: false,
        ),
      ).called(1);
    },
  );

  test('ltc26 MenuState equals works', () {
    final a = const MenuInitial();
    final b = const MenuInitial();
    expect(a, b, reason: 'MenuInitial instances should be equal');
  });

  test('ltc26 MenuEvent equals works', () {
    final a = MenuLockWalletPressed();
    final b = MenuLockWalletPressed();
    expect(a, b, reason: 'MenuLockWalletPressed instances should be equal');
  });
}
