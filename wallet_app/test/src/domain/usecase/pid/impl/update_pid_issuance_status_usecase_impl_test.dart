import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/domain/usecase/pid/impl/update_pid_issuance_status_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/pid/update_pid_issuance_status_usecase.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late UpdatePidIssuanceStatusUseCase usecase;
  final PidRepository mockRepo = Mocks.create();

  setUp(() {
    usecase = UpdatePidIssuanceStatusUseCaseImpl(mockRepo);
  });

  group('PidIssuanceEvent Status Updates', () {
    test('PidIssuanceEvent state should be passed on to the repository', () async {
      final event = PidIssuanceEvent.success(previewCards: List.empty());
      usecase.invoke(event);
      verify(mockRepo.notifyPidIssuanceStateUpdate(event));
    });
  });
}
