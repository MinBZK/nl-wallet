import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/wallet/impl/reset_wallet_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockWalletRepository mockWalletRepository;
  late MockTourRepository mockTourRepository;
  late ResetWalletUseCaseImpl resetWalletUseCase;

  setUp(() {
    mockWalletRepository = MockWalletRepository();
    mockTourRepository = MockTourRepository();
    resetWalletUseCase = ResetWalletUseCaseImpl(mockWalletRepository, mockTourRepository);
  });

  test('invoke calls walletRepository.reset', () async {
    // Act
    await resetWalletUseCase.invoke();

    // Verify that the reset method on the repositories was called exactly once.
    verify(mockWalletRepository.resetWallet()).called(1);
    verify(mockTourRepository.setShowTourBanner(showTourBanner: true)).called(1);
  });

  test('invoke completes successfully when walletRepository.reset completes', () async {
    // Act
    final future = resetWalletUseCase.invoke();

    // Assert
    // Expect that the future returned by invoke() completes without an error.
    await expectLater(future, completes);
  });

  test('invoke throws StateError when walletRepository.reset throws', () async {
    // Arrange
    when(mockWalletRepository.resetWallet()).thenThrow(Exception('Failed to reset wallet'));

    // Act
    final future = resetWalletUseCase.invoke();

    // Assert a StateError (which crashes the app) is thrown. As resetting is crucial and not allowed to fail.
    await expectLater(future, throwsA(isA<StateError>()));
  });
}
