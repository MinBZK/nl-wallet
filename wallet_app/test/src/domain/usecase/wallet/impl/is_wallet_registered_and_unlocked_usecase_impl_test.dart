import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/wallet/impl/is_wallet_registered_and_unlocked_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  test('When wallet is not registered and locked, usecase returns false', () async {
    final mockWalletRepository = MockWalletRepository();
    when(mockWalletRepository.isRegistered()).thenAnswer((_) async => false);
    when(mockWalletRepository.isLockedStream).thenAnswer((_) => Stream.value(true));
    final usecase = IsWalletRegisteredAndUnlockedUseCaseImpl(mockWalletRepository);
    final result = await usecase.invoke();
    expect(result, isFalse);
  });

  test('When wallet is registered and locked, usecase returns false', () async {
    final mockWalletRepository = MockWalletRepository();
    when(mockWalletRepository.isRegistered()).thenAnswer((_) async => true);
    when(mockWalletRepository.isLockedStream).thenAnswer((_) => Stream.value(true));
    final usecase = IsWalletRegisteredAndUnlockedUseCaseImpl(mockWalletRepository);
    final result = await usecase.invoke();
    expect(result, isFalse);
  });

  test('When wallet is registered and unlocked, usecase returns true', () async {
    final mockWalletRepository = MockWalletRepository();
    when(mockWalletRepository.isRegistered()).thenAnswer((_) async => true);
    when(mockWalletRepository.isLockedStream).thenAnswer((_) => Stream.value(false));
    final usecase = IsWalletRegisteredAndUnlockedUseCaseImpl(mockWalletRepository);
    final result = await usecase.invoke();
    expect(result, isTrue);
  });

  test('When wallet is not registered and unlocked, usecase returns false (this state should not occur in practice)',
      () async {
    final mockWalletRepository = MockWalletRepository();
    when(mockWalletRepository.isRegistered()).thenAnswer((_) async => false);
    when(mockWalletRepository.isLockedStream).thenAnswer((_) => Stream.value(false));
    final usecase = IsWalletRegisteredAndUnlockedUseCaseImpl(mockWalletRepository);
    final result = await usecase.invoke();
    expect(result, isFalse);
  });
}
