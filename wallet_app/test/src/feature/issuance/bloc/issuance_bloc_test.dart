import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_session_type.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_type.dart';
import 'package:wallet/src/domain/model/issuance/start_issuance_result.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/feature/issuance/bloc/issuance_bloc.dart';
import 'package:wallet/src/util/extension/string_extension.dart';

import '../../../mocks/wallet_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';

final _kDefaultReadyToDiscloseResponse = StartIssuanceReadyToDisclose(
  relyingParty: WalletMockData.organization,
  policy: WalletMockData.policy,
  sessionType: DisclosureSessionType.crossDevice,
  requestedAttributes: {},
  originUrl: 'https://example.org',
  requestPurpose: {},
  type: DisclosureType.regular,
  sharedDataWithOrganizationBefore: false,
);

void main() {
  final MockStartIssuanceUseCase startIssuanceUseCase = MockStartIssuanceUseCase();
  final MockCancelIssuanceUseCase cancelIssuanceUseCase = MockCancelIssuanceUseCase();

  setUp(() {
    provideDummy<Result<StartIssuanceResult>>(
      Result.success(_kDefaultReadyToDiscloseResponse),
    );
  });

  IssuanceBloc createBloc({bool isRefreshFlow = false}) => IssuanceBloc(
        startIssuanceUseCase,
        cancelIssuanceUseCase,
      );

  blocTest(
    'verify initial state',
    build: createBloc,
    verify: (bloc) => expect(bloc.state, const IssuanceInitial()),
  );

  blocTest(
    'IssuanceGenericError is emitted when issuance can not be initiated',
    build: () => createBloc(isRefreshFlow: true),
    setUp: () => when(startIssuanceUseCase.invoke(any))
        .thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test'))),
    act: (bloc) => bloc.add(const IssuanceInitiated('https://example.org')),
    expect: () => [isA<IssuanceGenericError>()],
  );

  blocTest(
    'verify happy path',
    build: () => createBloc(isRefreshFlow: false),
    setUp: () {
      when(startIssuanceUseCase.invoke(any)).thenAnswer(
        (_) async => Result.success(_kDefaultReadyToDiscloseResponse),
      );
    },
    act: (bloc) async {
      bloc.add(const IssuanceInitiated('https://example.org'));
      await Future.delayed(Duration(milliseconds: 10));
      bloc.add(const IssuanceOrganizationApproved());
      bloc.add(IssuancePinForDisclosureConfirmed(cards: [WalletMockData.altCard]));
      await Future.delayed(Duration(milliseconds: 10));
      bloc.add(IssuanceApproveCards(cards: [WalletMockData.altCard]));
      bloc.add(const IssuancePinForIssuanceConfirmed());
      await Future.delayed(Duration(milliseconds: 10));
    },
    expect: () => [
      isA<IssuanceCheckOrganization>(),
      isA<IssuanceProvidePinForDisclosure>(),
      isA<IssuanceLoadInProgress>(),
      isA<IssuanceReviewCards>(),
      isA<IssuanceProvidePinForIssuance>(),
      isA<IssuanceLoadInProgress>(),
      isA<IssuanceCompleted>().having((it) => it.addedCards, 'added cards should match', [WalletMockData.altCard]),
    ],
  );

  blocTest(
    'verify missing attributes path',
    build: () => createBloc(isRefreshFlow: false),
    setUp: () {
      when(startIssuanceUseCase.invoke(any)).thenAnswer(
        (_) async => Result.success(
          StartIssuanceMissingAttributes(
            relyingParty: WalletMockData.organization,
            sessionType: DisclosureSessionType.crossDevice,
            missingAttributes: [MissingAttribute(label: 'missing'.untranslated)],
            originUrl: 'https://example.org',
            requestPurpose: {},
            sharedDataWithOrganizationBefore: false,
          ),
        ),
      );
    },
    act: (bloc) async => bloc.add(const IssuanceInitiated('https://example.org')),
    expect: () => [
      isA<IssuanceMissingAttributes>().having(
        (state) => state.missingAttributes,
        'contains sample missing attribute',
        [MissingAttribute(label: 'missing'.untranslated)],
      ),
    ],
  );

  blocTest(
    'verify back press from confirm pin for disclosure',
    build: () => createBloc(isRefreshFlow: false),
    setUp: () {
      when(startIssuanceUseCase.invoke(any)).thenAnswer(
        (_) async => Result.success(_kDefaultReadyToDiscloseResponse),
      );
    },
    act: (bloc) async {
      bloc.add(const IssuanceInitiated('https://example.org'));
      await Future.delayed(Duration(milliseconds: 10));
      bloc.add(const IssuanceOrganizationApproved());
      bloc.add(const IssuanceBackPressed());
    },
    expect: () => [
      isA<IssuanceCheckOrganization>(),
      isA<IssuanceProvidePinForDisclosure>(),
      isA<IssuanceCheckOrganization>(),
    ],
  );

  blocTest(
    'verify back press from confirm pin for issuance',
    build: () => createBloc(isRefreshFlow: false),
    setUp: () {
      when(startIssuanceUseCase.invoke(any)).thenAnswer(
        (_) async => Result.success(_kDefaultReadyToDiscloseResponse),
      );
    },
    act: (bloc) async {
      bloc.add(const IssuanceInitiated('https://example.org'));
      await Future.delayed(Duration(milliseconds: 10));
      bloc.add(const IssuanceOrganizationApproved());
      bloc.add(IssuancePinForDisclosureConfirmed(cards: [WalletMockData.altCard]));
      await Future.delayed(Duration(milliseconds: 10));
      bloc.add(IssuanceApproveCards(cards: [WalletMockData.altCard]));
      bloc.add(const IssuanceBackPressed());
    },
    expect: () => [
      isA<IssuanceCheckOrganization>(),
      isA<IssuanceProvidePinForDisclosure>(),
      isA<IssuanceLoadInProgress>(),
      isA<IssuanceReviewCards>(),
      isA<IssuanceProvidePinForIssuance>(),
      isA<IssuanceReviewCards>(),
    ],
  );

  blocTest(
    'verify decline sharing attributes to organization path',
    build: () => createBloc(isRefreshFlow: false),
    setUp: () {
      when(startIssuanceUseCase.invoke(any)).thenAnswer(
        (_) async => Result.success(_kDefaultReadyToDiscloseResponse),
      );
    },
    act: (bloc) async {
      bloc.add(const IssuanceInitiated('https://example.org'));
      bloc.add(const IssuanceShareRequestedAttributesDeclined());
    },
    expect: () => [
      isA<IssuanceCheckOrganization>(),
      isA<IssuanceStopped>(),
    ],
  );

  blocTest(
    'verify disclosure failed with network error path',
    build: () => createBloc(isRefreshFlow: false),
    setUp: () {
      when(startIssuanceUseCase.invoke(any)).thenAnswer(
        (_) async => Result.success(_kDefaultReadyToDiscloseResponse),
      );
    },
    act: (bloc) async {
      bloc.add(const IssuanceInitiated('https://example.org'));
      await Future.delayed(Duration(milliseconds: 10));
      bloc.add(const IssuanceOrganizationApproved());
      bloc.add(const IssuanceConfirmPinFailed(error: NetworkError(hasInternet: true, sourceError: 'test')));
    },
    expect: () => [
      isA<IssuanceCheckOrganization>(),
      isA<IssuanceProvidePinForDisclosure>(),
      isA<IssuanceLoadInProgress>(),
      isA<IssuanceNetworkError>(),
    ],
  );

  blocTest(
    'verify issuance failed with generic error path',
    build: () => createBloc(isRefreshFlow: false),
    setUp: () {
      clearInteractions(cancelIssuanceUseCase);
      when(startIssuanceUseCase.invoke(any)).thenAnswer(
        (_) async => Result.success(_kDefaultReadyToDiscloseResponse),
      );
    },
    act: (bloc) async {
      bloc.add(const IssuanceInitiated('https://example.org'));
      await Future.delayed(Duration(milliseconds: 10));
      bloc.add(const IssuanceOrganizationApproved());
      bloc.add(IssuancePinForDisclosureConfirmed(cards: [WalletMockData.altCard]));
      await Future.delayed(Duration(milliseconds: 10));
      bloc.add(IssuanceApproveCards(cards: [WalletMockData.altCard]));
      bloc.add(const IssuanceConfirmPinFailed(error: GenericError('test', sourceError: 'test')));
    },
    expect: () => [
      isA<IssuanceCheckOrganization>(),
      isA<IssuanceProvidePinForDisclosure>(),
      isA<IssuanceLoadInProgress>(),
      isA<IssuanceReviewCards>(),
      isA<IssuanceProvidePinForIssuance>(),
      isA<IssuanceLoadInProgress>(),
      isA<IssuanceGenericError>(),
    ],
    verify: (_) {
      // once once for during initialization and once on error
      verify(cancelIssuanceUseCase.invoke()).called(2);
    },
  );

  blocTest(
    'verify path that contains a session error when trying to continue issuance',
    build: () => createBloc(isRefreshFlow: false),
    setUp: () {
      clearInteractions(cancelIssuanceUseCase);
      when(startIssuanceUseCase.invoke(any)).thenAnswer(
        (_) async => Result.success(_kDefaultReadyToDiscloseResponse),
      );
    },
    act: (bloc) async {
      bloc.add(const IssuanceInitiated('https://example.org'));
      await Future.delayed(Duration(milliseconds: 10));
      bloc.add(const IssuanceOrganizationApproved());
      bloc.add(IssuanceConfirmPinFailed(error: SessionError(state: SessionState.expired, sourceError: 'test')));
      await Future.delayed(Duration(milliseconds: 10));
    },
    expect: () => [
      isA<IssuanceCheckOrganization>(),
      isA<IssuanceProvidePinForDisclosure>(),
      isA<IssuanceLoadInProgress>(),
      isA<IssuanceSessionExpired>(),
    ],
  );

  blocTest(
    'verify path where session was cancelled externally leads to cancelled session state',
    build: () => createBloc(isRefreshFlow: false),
    setUp: () {
      clearInteractions(cancelIssuanceUseCase);
      when(startIssuanceUseCase.invoke(any)).thenAnswer(
        (_) async => Result.success(_kDefaultReadyToDiscloseResponse),
      );
    },
    act: (bloc) async {
      bloc.add(const IssuanceInitiated('https://example.org'));
      await Future.delayed(Duration(milliseconds: 10));
      bloc.add(const IssuanceOrganizationApproved());
      bloc.add(IssuanceConfirmPinFailed(error: SessionError(state: SessionState.cancelled, sourceError: 'test')));
      await Future.delayed(Duration(milliseconds: 10));
    },
    expect: () => [
      isA<IssuanceCheckOrganization>(),
      isA<IssuanceProvidePinForDisclosure>(),
      isA<IssuanceLoadInProgress>(),
      isA<IssuanceSessionCancelled>(),
    ],
  );

  blocTest(
    'verify accepting zero cards results in error',
    build: () => createBloc(isRefreshFlow: false),
    setUp: () {
      when(startIssuanceUseCase.invoke(any)).thenAnswer(
        (_) async => Result.success(_kDefaultReadyToDiscloseResponse),
      );
    },
    act: (bloc) async {
      bloc.add(const IssuanceInitiated('https://example.org'));
      await Future.delayed(Duration(milliseconds: 10));
      bloc.add(const IssuanceOrganizationApproved());
      bloc.add(IssuancePinForDisclosureConfirmed(cards: [WalletMockData.altCard]));
      await Future.delayed(Duration(milliseconds: 10));
      bloc.add(IssuanceApproveCards(cards: []));
    },
    expect: () => [
      isA<IssuanceCheckOrganization>(),
      isA<IssuanceProvidePinForDisclosure>(),
      isA<IssuanceLoadInProgress>(),
      isA<IssuanceReviewCards>(),
      isA<IssuanceGenericError>(),
    ],
  );
}
