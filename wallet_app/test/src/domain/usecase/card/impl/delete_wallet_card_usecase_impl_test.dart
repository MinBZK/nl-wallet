import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/card/delete_wallet_card_usecase.dart';
import 'package:wallet/src/domain/usecase/card/impl/delete_wallet_card_usecase_impl.dart';
import 'package:wallet_core/core.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockWalletCardRepository mockRepository;
  late DeleteWalletCardUseCase usecase;

  const attestationId = 'card-123';

  setUp(() {
    mockRepository = MockWalletCardRepository();
    usecase = DeleteWalletCardUseCaseImpl(mockRepository, attestationId);
  });

  test('invoke calls delete with correct parameters', () async {
    const pin = '123456';
    when(mockRepository.delete(any, any)).thenAnswer((_) async {});

    await usecase.invoke(pin);

    verify(mockRepository.delete(pin, attestationId));
  });

  test('invoke returns success when delete completes without errors', () async {
    const pin = '123456';
    when(mockRepository.delete(any, any)).thenAnswer((_) async {});

    final result = await usecase.invoke(pin);

    expect(result.hasError, isFalse);
  });

  test('invoke returns CheckPinError when delete throws WalletInstructionError', () async {
    const pin = '000000';
    when(mockRepository.delete(any, any)).thenThrow(
      const WalletInstructionError.incorrectPin(attemptsLeftInRound: 2, isFinalRound: false),
    );

    final result = await usecase.invoke(pin);

    expect(result.hasError, isTrue);
    expect(result.error, isA<CheckPinError>());
  });
}
