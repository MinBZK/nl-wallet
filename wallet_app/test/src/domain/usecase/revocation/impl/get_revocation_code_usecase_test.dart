import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/revocation/impl/get_revocation_code_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockRevocationRepository revocationRepository;
  late GetRevocationCodeUseCaseImpl useCase;

  setUp(() {
    revocationRepository = MockRevocationRepository();
    useCase = GetRevocationCodeUseCaseImpl(revocationRepository);
  });

  test('invoke returns success with code', () async {
    const code = '123456';
    when(revocationRepository.getRevocationCode(any)).thenAnswer((_) async => code);

    final result = await useCase.invoke('1234');

    expect(result.value, code);
    verify(revocationRepository.getRevocationCode('1234')).called(1);
  });

  test('invoke returns error when repository fails', () async {
    when(revocationRepository.getRevocationCode(any)).thenThrow(Exception());

    final result = await useCase.invoke('1234');

    expect(result.hasError, true);
    verify(revocationRepository.getRevocationCode('1234')).called(1);
  });
}
