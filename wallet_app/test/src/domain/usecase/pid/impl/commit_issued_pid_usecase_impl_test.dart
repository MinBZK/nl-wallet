import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/domain/usecase/pid/commit_issued_pid_usecase.dart';
import 'package:wallet/src/domain/usecase/pid/impl/commit_issued_pid_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late CommitIssuedPidUseCase usecase;
  late MockPidRepository mockRepo;

  setUp(() {
    mockRepo = Mocks.create<PidRepository>() as MockPidRepository;
    usecase = CommitIssuedPidUseCaseImpl(mockRepo);
  });

  test('the transfer state is returned after committing the pid', () async {
    when(mockRepo.commitIssuedPid('123123')).thenAnswer((_) async => TransferState.available);

    final result = await usecase.invoke('123123');

    expect(result.hasError, isFalse);
    expect(result.value, TransferState.available);
    verify(mockRepo.commitIssuedPid('123123')).called(1);
  });

  test('errors from the repository are returned as error result', () async {
    when(mockRepo.commitIssuedPid('123123')).thenAnswer((_) async => throw Exception('error'));

    final result = await usecase.invoke('123123');

    expect(result.hasError, isTrue);
  });
}
