import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/issuance/continue_issuance_result.dart';
import 'package:wallet/src/domain/model/issuance/start_issuance_result.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/feature/issuance/bloc/issuance_bloc.dart';

import '../../../mocks/wallet_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';

void main() {
  final MockStartIssuanceUseCase startIssuanceUseCase = MockStartIssuanceUseCase();
  final MockContinueIssuanceUseCase continueIssuanceUseCase = MockContinueIssuanceUseCase();
  final MockAcceptIssuanceUseCase acceptIssuanceUseCase = MockAcceptIssuanceUseCase();

  setUp(() {
    provideDummy<Result<ContinueIssuanceResult>>(Result.success(ContinueIssuanceResult([])));
    provideDummy<Result<StartIssuanceResult>>(
      Result.success(
        StartIssuanceReadyToDisclose(
          relyingParty: WalletMockData.organization,
          policy: WalletMockData.policy,
          requestedAttributes: {},
        ),
      ),
    );
  });

  IssuanceBloc createBloc({bool isRefreshFlow = false}) => IssuanceBloc(
        startIssuanceUseCase,
        continueIssuanceUseCase,
        acceptIssuanceUseCase,
        Mocks.create(),
        isRefreshFlow: isRefreshFlow,
      );

  blocTest(
    'verify initial state',
    build: createBloc,
    verify: (bloc) => expect(bloc.state, const IssuanceInitial()),
  );

  blocTest(
    'IssuanceGenericError is emitted when issuance can not be initiated, and correct isRefreshFlow flag is reported',
    build: () => createBloc(isRefreshFlow: true),
    setUp: () => when(startIssuanceUseCase.invoke(any))
        .thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test'))),
    act: (bloc) => bloc.add(const IssuanceInitiated('https://example.org')),
    verify: (bloc) => expect(bloc.state.isRefreshFlow, isTrue),
    expect: () => [isA<IssuanceGenericError>()],
  );

  blocTest(
    'verify happy path',
    build: () => createBloc(isRefreshFlow: false),
    setUp: () {
      when(startIssuanceUseCase.invoke(any)).thenAnswer(
        (_) async => Result.success(
          StartIssuanceReadyToDisclose(
            relyingParty: WalletMockData.organization,
            policy: WalletMockData.policy,
            requestedAttributes: {},
          ),
        ),
      );
      when(continueIssuanceUseCase.invoke())
          .thenAnswer((_) async => Result.success(ContinueIssuanceResult([WalletMockData.altCard])));
      when(acceptIssuanceUseCase.invoke(any)).thenAnswer((_) async => const Result.success(null));
    },
    act: (bloc) {
      bloc.add(const IssuanceInitiated('https://example.org'));
      bloc.add(const IssuanceOrganizationApproved());
      bloc.add(const IssuanceShareRequestedAttributesApproved());
      bloc.add(const IssuancePinConfirmed());
      bloc.add(const IssuanceCheckDataOfferingApproved());
    },
    verify: (bloc) => expect(bloc.state.isRefreshFlow, isFalse),
    expect: () => [
      isA<IssuanceCheckOrganization>(),
      isA<IssuanceProofIdentity>(),
      isA<IssuanceProvidePin>(),
      isA<IssuanceCheckDataOffering>(),
      isA<IssuanceCompleted>(),
    ],
  );
}
