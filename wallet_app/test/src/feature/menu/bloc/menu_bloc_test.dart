import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/feature/menu/bloc/menu_bloc.dart';

import '../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockLockWalletUseCase lockUsecase;

  setUp(() {
    lockUsecase = MockLockWalletUseCase();
  });

  blocTest(
    'verify initial state',
    build: () => MenuBloc(lockUsecase),
    verify: (bloc) => expect(bloc.state, const MenuInitial()),
  );

  blocTest(
    'MenuLockWalletPressed invokes the lock usecase',
    build: () => MenuBloc(lockUsecase),
    act: (bloc) => bloc.add(MenuLockWalletPressed()),
    verify: (bloc) => verify(lockUsecase.invoke()).called(1),
  );

  test('MenuState equals works', () {
    final a = MenuInitial();
    final b = MenuInitial();
    expect(a, b, reason: 'MenuInitial instances should be equal');
  });

  test('MenuEvent equals works', () {
    final a = MenuLockWalletPressed();
    final b = MenuLockWalletPressed();
    expect(a, b, reason: 'MenuLockWalletPressed instances should be equal');
  });
}
