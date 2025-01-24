import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/usecase/pid/continue_pid_issuance_usecase.dart';
import 'package:wallet/src/domain/usecase/pid/impl/continue_pid_issuance_usecase_impl.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late ContinuePidIssuanceUseCase usecase;
  final mockRepo = Mocks.create<PidRepository>() as MockPidRepository;

  setUp(() {
    usecase = ContinuePidIssuanceUseCaseImpl(mockRepo);
  });

  group('PidIssuance Status Verification', () {
    test('PidIssuanceSuccess is emitted with the sample attributes', () async {
      final sampleAttribute = DataAttribute.untranslated(
        key: 'key',
        label: 'label',
        value: const StringValue('value'),
        sourceCardDocType: 'sourceCardId',
      );
      const samplePidIssuanceUri = 'https://example.org';
      when(mockRepo.continuePidIssuance(samplePidIssuanceUri)).thenAnswer((_) async => [sampleAttribute]);

      final result = await usecase.invoke(samplePidIssuanceUri);
      expect(result, PidIssuanceSuccess([sampleAttribute]));
    });

    test('PidIssuanceError is emitted with the thrown redirectError', () async {
      const samplePidIssuanceUri = 'https://example.org';
      when(mockRepo.continuePidIssuance(samplePidIssuanceUri)).thenAnswer(
        (_) async => throw const CoreRedirectUriError(
          'expected error',
          redirectError: RedirectError.accessDenied,
        ),
      );

      final result = await usecase.invoke(samplePidIssuanceUri);
      expect(result, PidIssuanceError(RedirectError.accessDenied));
    });
  });
}
