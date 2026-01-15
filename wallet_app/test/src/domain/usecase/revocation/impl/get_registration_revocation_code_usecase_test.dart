import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/revocation/impl/get_registration_revocation_code_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockRevocationRepository revocationRepository;
  late GetRegistrationRevocationCodeUseCaseImpl useCase;

  setUp(() {
    revocationRepository = MockRevocationRepository();
    useCase = GetRegistrationRevocationCodeUseCaseImpl(revocationRepository);
  });

  test('invoke returns success with code', () async {
    const code = '123456';
    when(revocationRepository.getRegistrationRevocationCode()).thenAnswer((_) async => code);

    final result = await useCase.invoke();

    expect(result.value, code);
    verify(revocationRepository.getRegistrationRevocationCode()).called(1);
  });

  test('invoke returns error when repository fails', () async {
    when(revocationRepository.getRegistrationRevocationCode()).thenThrow(Exception());

    final result = await useCase.invoke();

    expect(result.hasError, true);
    verify(revocationRepository.getRegistrationRevocationCode()).called(1);
  });
}
