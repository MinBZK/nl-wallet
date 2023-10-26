import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/domain/model/attribute/data_attribute.dart';
import 'package:wallet/src/domain/usecase/pid/continue_pid_issuance_usecase.dart';
import 'package:wallet/src/domain/usecase/pid/impl/continue_pid_issuance_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late ContinuePidIssuanceUseCase usecase;
  final mockRepo = Mocks.create<PidRepository>() as MockPidRepository;
  late BehaviorSubject<List<DataAttribute>> mockPidPreviewAttributes;

  setUp(() {
    mockPidPreviewAttributes = BehaviorSubject<List<DataAttribute>>();
    when(mockRepo.continuePidIssuance(Uri.parse('https://example.org'))).thenAnswer((_) => mockPidPreviewAttributes);
    usecase = ContinuePidIssuanceUseCaseImpl(mockRepo);
  });

  group('PidIssuance Status Verification', () {
    test('PidIssuanceSuccess is emitted with the sample attributes', () async {
      const sampleAttribute = DataAttribute(
          key: 'key', label: 'label', value: 'value', sourceCardId: 'sourceCardId', valueType: AttributeValueType.text);
      expectLater(
        usecase.invoke(Uri.parse('https://example.org')),
        emitsInOrder(
          [
            PidIssuanceAuthenticating(),
            PidIssuanceSuccess(const [sampleAttribute])
          ],
        ),
      );
      // Causes the mock repository to expose these as preview attributes
      mockPidPreviewAttributes.add([sampleAttribute]);
    });
  });
}
