import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/usecase/revocation/get_registration_revocation_code_usecase.dart';
import 'package:wallet/src/domain/usecase/revocation/set_revocation_code_saved_usecase.dart';
import 'package:wallet/src/feature/revocation/bloc/revocation_code_bloc.dart';

import '../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late SetRevocationCodeSavedUseCase setRevocationCodeSavedUseCase;
  late GetRegistrationRevocationCodeUseCase getRegistrationRevocationCodeUseCase;

  setUp(() {
    setRevocationCodeSavedUseCase = MockSetRevocationCodeSavedUseCase();
    getRegistrationRevocationCodeUseCase = MockGetRegistrationRevocationCodeUseCase();
  });

  blocTest<RevocationCodeBloc, RevocationCodeState>(
    'verify initial state',
    build: () => RevocationCodeBloc(
      setRevocationCodeSavedUseCase,
      getRegistrationRevocationCodeUseCase,
    ),
    verify: (bloc) => expect(bloc.state, const RevocationCodeInitial()),
  );

  blocTest<RevocationCodeBloc, RevocationCodeState>(
    'verify RevocationCodeLoadSuccess state when code is fetched',
    build: () => RevocationCodeBloc(
      setRevocationCodeSavedUseCase,
      getRegistrationRevocationCodeUseCase,
    ),
    setUp: () {
      when(getRegistrationRevocationCodeUseCase.invoke()).thenAnswer(
        (_) async => const Result.success('123456'),
      );
    },
    act: (bloc) => bloc.add(const RevocationCodeLoadTriggered()),
    expect: () => [const RevocationCodeLoadSuccess('123456')],
  );

  blocTest<RevocationCodeBloc, RevocationCodeState>(
    'verify RevocationCodeSaveSuccess state when continue is pressed',
    build: () => RevocationCodeBloc(
      setRevocationCodeSavedUseCase,
      getRegistrationRevocationCodeUseCase,
    ),
    setUp: () {
      when(getRegistrationRevocationCodeUseCase.invoke()).thenAnswer(
        (_) async => const Result.success('123456'),
      );
      when(setRevocationCodeSavedUseCase.invoke(saved: true)).thenAnswer(
        (_) async => const Result.success(null),
      );
    },
    act: (bloc) async {
      bloc.add(const RevocationCodeLoadTriggered());
      await Future.delayed(Duration.zero);
      bloc.add(const RevocationCodeContinuePressed());
    },
    expect: () => [
      const RevocationCodeLoadSuccess('123456'),
      const RevocationCodeSaveSuccess('123456'),
    ],
    verify: (bloc) {
      verify(setRevocationCodeSavedUseCase.invoke(saved: true)).called(1);
    },
  );
}
