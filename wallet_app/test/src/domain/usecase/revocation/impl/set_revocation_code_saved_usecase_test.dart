import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/revocation/impl/set_revocation_code_saved_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockRevocationRepository revocationRepository;
  late SetRevocationCodeSavedUseCaseImpl useCase;

  setUp(() {
    revocationRepository = MockRevocationRepository();
    useCase = SetRevocationCodeSavedUseCaseImpl(revocationRepository);
  });

  test('invoke calls repository with correct parameters', () async {
    const saved = true;
    when(revocationRepository.setRevocationCodeSaved(saved: saved)).thenAnswer((_) async {});

    await useCase.invoke(saved: saved);

    verify(revocationRepository.setRevocationCodeSaved(saved: saved)).called(1);
  });

  test('invoke returns error when repository fails', () async {
    const saved = true;
    when(revocationRepository.setRevocationCodeSaved(saved: saved)).thenThrow(Exception());

    final result = await useCase.invoke(saved: saved);

    expect(result.hasError, true);
    verify(revocationRepository.setRevocationCodeSaved(saved: saved)).called(1);
  });
}
