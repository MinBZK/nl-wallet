import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/issuance/issuance_repository.dart';
import 'package:wallet/src/domain/model/issuance/start_issuance_result.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/issuance/impl/start_issuance_usecase_impl.dart';
import 'package:wallet/src/feature/issuance/argument/issuance_screen_argument.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late StartIssuanceUseCaseImpl usecase;
  final mockRepo = Mocks.create<IssuanceRepository>() as MockIssuanceRepository;

  setUp(() {
    usecase = StartIssuanceUseCaseImpl(mockRepo);
  });

  group('StartIssuanceUseCaseImpl', () {
    const sampleUri = 'https://example.org/issuance';

    test('Success: calls startIssuance when type is disclosureBasedIssuance', () async {
      const resultValue = StartIssuanceResult.authorizationRequired('https://auth.url');
      when(mockRepo.startIssuance(sampleUri, isQrCode: true)).thenAnswer((_) async => resultValue);

      final result = await usecase.invoke(sampleUri, type: IssuanceType.disclosureBasedIssuance, isQrCode: true);

      expect(result.hasError, isFalse);
      expect(result.value, resultValue);
      verify(mockRepo.startIssuance(sampleUri, isQrCode: true)).called(1);
    });

    test('Success: calls startIssuanceFromOffer when type is credentialOffer', () async {
      const resultValue = StartIssuanceResult.authorizationRequired('https://auth.url');
      when(mockRepo.startIssuanceFromOffer(sampleUri, isQrCode: false)).thenAnswer((_) async => resultValue);

      final result = await usecase.invoke(sampleUri, type: IssuanceType.credentialOffer, isQrCode: false);

      expect(result.hasError, isFalse);
      expect(result.value, resultValue);
      verify(mockRepo.startIssuanceFromOffer(sampleUri, isQrCode: false)).called(1);
    });

    test('Failure: returns error when repo throws CoreError', () async {
      when(mockRepo.startIssuance(sampleUri, isQrCode: false)).thenThrow(
        const CoreGenericError('something went wrong'),
      );

      final result = await usecase.invoke(sampleUri, type: IssuanceType.disclosureBasedIssuance);

      expect(result.hasError, isTrue);
      expect(result.error, isA<GenericError>());
      verify(mockRepo.startIssuance(sampleUri, isQrCode: false)).called(1);
    });

    test('Failure: returns error when type is authorizationCallback', () async {
      final result = await usecase.invoke(sampleUri, type: IssuanceType.authorizationCallback);

      expect(result.hasError, isTrue);
      expect(result.error, isA<GenericError>());
      expect(
        (result.error! as GenericError).rawMessage,
        contains('authorizationCallback should rely on continueIssuance()'),
      );
    });
  });
}
