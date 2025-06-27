import 'package:bloc_test/bloc_test.dart';
import 'package:mockito/mockito.dart';
import 'package:test/test.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/model/start_sign_result/start_sign_result.dart';
import 'package:wallet/src/feature/sign/bloc/sign_bloc.dart';

import '../../../mocks/wallet_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';

// Currently signing is only supported by the mock build, only testing the bloc behaviour superficially.
void main() {
  late MockStartSignUseCase startSignUseCase;
  late MockRejectSignAgreementUseCase rejectSignAgreementUseCase;

  /// Create a new [SignBloc] configured with the (mocked) usecases
  SignBloc create() => SignBloc('uri', startSignUseCase, rejectSignAgreementUseCase);

  setUp(() {
    startSignUseCase = MockStartSignUseCase();
    rejectSignAgreementUseCase = MockRejectSignAgreementUseCase();
    provideDummy<Result<StartSignResult>>(
      Result.success(
        StartSignReadyToSign(
          document: WalletMockData.document,
          policy: WalletMockData.policy,
          relyingParty: WalletMockData.organization,
          trustProvider: WalletMockData.organization,
          requestedCards: [],
        ),
      ),
    );
  });

  test('initial state is correct', () {
    expect(create().state, const SignLoadInProgress());
  });

  blocTest(
    'when startSignUseCase fails, emit generic error',
    setUp: () => when(startSignUseCase.invoke(any))
        .thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test'))),
    build: create,
    expect: () => [isA<SignError>()],
  );

  blocTest(
    'verify happy flow to up to SignSuccess',
    setUp: () => when(startSignUseCase.invoke(any)).thenAnswer((_) async {
      return Result.success(
        StartSignReadyToSign(
          document: WalletMockData.document,
          policy: WalletMockData.policy,
          relyingParty: WalletMockData.organization,
          trustProvider: WalletMockData.organization,
          requestedCards: [],
        ),
      );
    }),
    build: create,
    act: (bloc) async {
      await Future.delayed(const Duration(milliseconds: 25));
      bloc.add(const SignOrganizationApproved());
      await Future.delayed(const Duration(milliseconds: 25));
      bloc.add(const SignAgreementChecked());
      await Future.delayed(const Duration(milliseconds: 25));
      bloc.add(const SignAgreementApproved());
      await Future.delayed(const Duration(milliseconds: 25));
      bloc.add(const SignPinConfirmed());
    },
    expect: () async => [
      isA<SignCheckOrganization>(),
      isA<SignCheckAgreement>(),
      isA<SignConfirmAgreement>(),
      isA<SignConfirmPin>(),
      isA<SignLoadInProgress>(),
      isA<SignSuccess>(),
    ],
  );
}
