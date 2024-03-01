import 'package:mockito/mockito.dart';
import 'package:test/test.dart';
import 'package:wallet/src/domain/usecase/disclosure/cancel_disclosure_usecase.dart';
import 'package:wallet/src/domain/usecase/disclosure/impl/cancel_disclosure_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  final repository = MockDisclosureRepository();
  late CancelDisclosureUseCase usecase;

  setUp(() {
    usecase = CancelDisclosureUseCaseImpl(repository);
  });

  test('cancel is called when session is active', () async {
    when(repository.hasActiveDisclosureSession()).thenAnswer((_) async => true);
    await usecase.invoke();
    verify(repository.cancelDisclosure()).called(1);
  });

  test('cancel is not called when session is not active', () async {
    when(repository.hasActiveDisclosureSession()).thenAnswer((_) async => false);
    await usecase.invoke();
    verifyNever(repository.cancelDisclosure());
  });
}
