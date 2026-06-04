import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/wallet_state.dart';
import 'package:wallet/src/domain/usecase/session/cancel_session_usecase.dart';
import 'package:wallet/src/domain/usecase/session/impl/cancel_session_usecase_impl.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late CancelSessionUseCase usecase;
  final mockWalletRepository = MockWalletRepository();

  setUp(() {
    usecase = CancelSessionUseCaseImpl(mockWalletRepository);
    reset(mockWalletRepository);
  });

  group('CancelActiveSessionUseCase', () {
    test('calls cancelActiveSession when state is WalletStateInDisclosureFlow', () async {
      when(mockWalletRepository.getWalletState()).thenAnswer((_) async => const WalletStateInDisclosureFlow());
      when(mockWalletRepository.cancelSession()).thenAnswer((_) async => 'https://return.url');

      final result = await usecase.invoke();

      verify(mockWalletRepository.cancelSession()).called(1);
      expect(result.hasError, isFalse);
      expect(result.value, 'https://return.url');
    });

    test('calls cancelActiveSession when state is WalletStateInIssuanceFlow', () async {
      when(mockWalletRepository.getWalletState()).thenAnswer((_) async => const WalletStateInIssuanceFlow());
      when(mockWalletRepository.cancelSession()).thenAnswer((_) async => 'https://return.url');

      final result = await usecase.invoke();

      verify(mockWalletRepository.cancelSession()).called(1);
      expect(result.hasError, isFalse);
      expect(result.value, 'https://return.url');
    });

    test('does not call cancelActiveSession when state is WalletStateReady', () async {
      when(mockWalletRepository.getWalletState()).thenAnswer((_) async => const WalletStateReady());

      final result = await usecase.invoke();

      verifyNever(mockWalletRepository.cancelSession());
      expect(result.hasError, isFalse);
      expect(result.value, isNull);
    });

    test('returns returnUrl from CoreError when available', () async {
      when(mockWalletRepository.getWalletState()).thenAnswer((_) async => const WalletStateInDisclosureFlow());
      when(mockWalletRepository.cancelSession()).thenThrow(
        const CoreGenericError('error', data: {'return_url': 'https://error.return.url'}),
      );

      final result = await usecase.invoke();

      expect(result.hasError, isFalse);
      expect(result.value, 'https://error.return.url');
    });

    test('returns error from CoreError when returnUrl is NOT available', () async {
      when(mockWalletRepository.getWalletState()).thenAnswer((_) async => const WalletStateInDisclosureFlow());
      when(mockWalletRepository.cancelSession()).thenThrow(
        const CoreGenericError('error'),
      );

      final result = await usecase.invoke();

      expect(result.hasError, isTrue);
      expect(result.error, isA<GenericError>());
    });

    test('returns GenericError for unknown exceptions', () async {
      when(mockWalletRepository.getWalletState()).thenAnswer((_) async => const WalletStateInDisclosureFlow());
      when(mockWalletRepository.cancelSession()).thenThrow(Exception('random error'));

      final result = await usecase.invoke();

      expect(result.hasError, isTrue);
      expect(result.error, isA<GenericError>());
    });
  });
}
