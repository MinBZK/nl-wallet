import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/domain/usecase/pid/impl/observe_pid_issuance_status_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/pid/observe_pid_issuance_status_usecase.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late ObservePidIssuanceStatusUseCase usecase;
  final mockRepo = Mocks.create<PidRepository>() as MockPidRepository;
  late BehaviorSubject<PidIssuanceStatus> mockStatusSubject;

  setUp(() {
    mockStatusSubject = BehaviorSubject<PidIssuanceStatus>();
    when(mockRepo.observePidIssuanceStatus()).thenAnswer((_) => mockStatusSubject);
    usecase = ObservePidIssuanceStatusUseCaseImpl(mockRepo);
  });

  group('PidIssuance Status Verification', () {
    test('Nothing should be emitted when the repo is not notified', () async {
      expectLater(usecase.invoke(), emitsInOrder([]));
    });

    test('The stream should be updated after notifying the repository', () async {
      expectLater(usecase.invoke(), emitsInOrder([PidIssuanceAuthenticating()]));
      mockStatusSubject.add(PidIssuanceAuthenticating());
    });

    test('The idle state is expected to be consumed by the usecase', () async {
      expectLater(usecase.invoke(), emitsInOrder([PidIssuanceAuthenticating(), PidIssuanceSuccess(List.empty())]));
      mockStatusSubject.add(PidIssuanceAuthenticating());
      mockStatusSubject.add(PidIssuanceSuccess(List.empty()));
      mockStatusSubject.add(PidIssuanceIdle());
    });
  });
}
