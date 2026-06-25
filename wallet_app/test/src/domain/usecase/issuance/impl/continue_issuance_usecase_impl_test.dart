import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/issuance/issuance_repository.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/issuance/continue_issuance_usecase.dart';
import 'package:wallet/src/domain/usecase/issuance/impl/continue_issuance_usecase_impl.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late ContinueIssuanceUseCase usecase;
  final mockRepo = Mocks.create<IssuanceRepository>() as MockIssuanceRepository;

  setUp(() {
    usecase = ContinueIssuanceUseCaseImpl(mockRepo);
  });

  group('ContinueIssuanceUseCase', () {
    test('Success: returns a list of cards when repo succeeds', () async {
      const sampleUri = 'https://example.org';
      final sampleCards = [WalletMockData.card];
      when(mockRepo.continueIssuance(sampleUri)).thenAnswer((_) async => sampleCards);

      final result = await usecase.invoke(sampleUri);

      expect(result.hasError, isFalse);
      expect(result.value, sampleCards);
      verify(mockRepo.continueIssuance(sampleUri)).called(1);
    });

    test('Failure: returns an ApplicationError when repo throws', () async {
      const sampleUri = 'https://example.org';
      when(mockRepo.continueIssuance(sampleUri)).thenThrow(
        const CoreRedirectUriError(
          'access denied',
          redirectError: RedirectError.accessDenied,
        ),
      );

      final result = await usecase.invoke(sampleUri);

      expect(result.hasError, isTrue);
      expect(result.error, isA<RedirectUriError>());
      final redirectError = result.error! as RedirectUriError;
      expect(redirectError.redirectError, RedirectError.accessDenied);
      verify(mockRepo.continueIssuance(sampleUri)).called(1);
    });
  });
}
