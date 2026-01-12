import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/revocation/impl/get_revocation_code_saved_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockRevocationRepository revocationRepository;
  late GetRevocationCodeSavedUseCaseImpl useCase;

  setUp(() {
    revocationRepository = MockRevocationRepository();
    useCase = GetRevocationCodeSavedUseCaseImpl(revocationRepository);
  });

  test('invoke returns success with saved flag', () async {
    const saved = true;
    when(revocationRepository.getRevocationCodeSaved()).thenAnswer((_) async => saved);

    final result = await useCase.invoke();

    expect(result.value, saved);
    verify(revocationRepository.getRevocationCodeSaved()).called(1);
  });

  test('invoke returns error when repository fails', () async {
    when(revocationRepository.getRevocationCodeSaved()).thenThrow(Exception());

    final result = await useCase.invoke();

    expect(result.hasError, true);
    verify(revocationRepository.getRevocationCodeSaved()).called(1);
  });
}
