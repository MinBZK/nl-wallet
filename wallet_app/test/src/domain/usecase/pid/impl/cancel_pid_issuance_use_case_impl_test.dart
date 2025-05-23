import 'package:mockito/mockito.dart';
import 'package:test/test.dart';
import 'package:wallet/src/domain/usecase/pid/cancel_pid_issuance_usecase.dart';
import 'package:wallet/src/domain/usecase/pid/impl/cancel_pid_issuance_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  final pidRepository = MockPidRepository();
  late CancelPidIssuanceUseCase usecase;

  setUp(() {
    usecase = CancelPidIssuanceUseCaseImpl(pidRepository);
  });

  test('cancel is called when session is active', () async {
    when(pidRepository.hasActiveIssuanceSession()).thenAnswer((_) async => true);
    await usecase.invoke();
    verify(pidRepository.cancelIssuance()).called(1);
  });

  test('cancel is not called when session is not active', () async {
    when(pidRepository.hasActiveIssuanceSession()).thenAnswer((_) async => false);
    await usecase.invoke();
    verifyNever(pidRepository.cancelIssuance());
  });
}
