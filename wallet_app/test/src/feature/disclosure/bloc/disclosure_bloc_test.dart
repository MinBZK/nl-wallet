import 'package:bloc_test/bloc_test.dart';
import 'package:mockito/mockito.dart';
import 'package:test/test.dart';
import 'package:wallet/src/data/repository/disclosure/disclosure_repository.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_session_type.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_type.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/feature/disclosure/bloc/disclosure_bloc.dart';
import 'package:wallet/src/feature/report_issue/reporting_option.dart';
import 'package:wallet/src/util/extension/core_error_extension.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../mocks/wallet_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockStartDisclosureUseCase startDisclosureUseCase;
  late MockCancelDisclosureUseCase cancelDisclosureUseCase;
  late MockGetMostRecentWalletEventUseCase getMostRecentWalletEventUsecase;

  /// Create a new [DisclosureBloc] configured with the (mocked) usecases
  DisclosureBloc create() =>
      DisclosureBloc(startDisclosureUseCase, cancelDisclosureUseCase, getMostRecentWalletEventUsecase);

  setUp(() {
    startDisclosureUseCase = MockStartDisclosureUseCase();
    cancelDisclosureUseCase = MockCancelDisclosureUseCase();
    getMostRecentWalletEventUsecase = MockGetMostRecentWalletEventUseCase();
    provideDummy<Result<StartDisclosureResult>>(
      Result.success(
        StartDisclosureReadyToDisclose(
          relyingParty: WalletMockData.organization,
          originUrl: 'http://origin.org',
          requestPurpose: 'requestPurpose'.untranslated,
          sessionType: DisclosureSessionType.crossDevice,
          type: DisclosureType.login,
          policy: WalletMockData.policy,
          sharedDataWithOrganizationBefore: false,
          cardRequests: [],
        ),
      ),
    );
  });

  test('initial state is correct', () {
    expect(create().state, const DisclosureInitial());
  });

  blocTest(
    'when startDisclosure fails, emit generic error',
    setUp: () => when(
      startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode')),
    ).thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test'))),
    build: create,
    act: (bloc) => bloc.add(const DisclosureSessionStarted('')),
    expect: () => [
      isA<DisclosureLoadInProgress>(),
      isA<DisclosureGenericError>(),
    ],
  );

  blocTest(
    'when startDisclosure fails with network issue, emit DisclosureNetworkError(hasInternet: true)',
    setUp: () => when(
      startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode')),
    ).thenAnswer((_) async => const Result.error(NetworkError(hasInternet: true, sourceError: 'test'))),
    build: create,
    act: (bloc) => bloc.add(const DisclosureSessionStarted('')),
    verify: (bloc) {
      expect(bloc.state, isA<DisclosureNetworkError>());
      expect((bloc.state as DisclosureNetworkError).hasInternet, isTrue);
      expect((bloc.state as DisclosureNetworkError).error, isA<NetworkError>());
    },
  );

  blocTest(
    'when startDisclosure fails with network issue and there is no internet, emit DisclosureNetworkError(hasInternet: false)',
    setUp: () {
      when(
        startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode')),
      ).thenAnswer((_) async => const Result.error(NetworkError(hasInternet: false, sourceError: 'test')));
      when(CoreErrorExtension.networkRepository.hasInternet()).thenAnswer((realInvocation) async {
        await Future.delayed(const Duration(milliseconds: 100));
        return false;
      });
    },
    wait: const Duration(milliseconds: 150),
    build: create,
    act: (bloc) => bloc.add(const DisclosureSessionStarted('')),
    verify: (bloc) {
      expect(bloc.state, isA<DisclosureNetworkError>());
      expect((bloc.state as DisclosureNetworkError).hasInternet, isFalse);
      expect((bloc.state as DisclosureNetworkError).error, isA<NetworkError>());
    },
  );

  blocTest(
    'when startDisclosure returns StartDisclosureReadyToDisclose for regular disclosure, the bloc emits DisclosureCheckOrganization',
    setUp: () {
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureReadyToDisclose(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'requestPurpose'.untranslated,
            sessionType: DisclosureSessionType.crossDevice,
            type: DisclosureType.regular,
            policy: WalletMockData.policy,
            sharedDataWithOrganizationBefore: false,
            cardRequests: [],
          ),
        );
      });
    },
    build: create,
    act: (bloc) => bloc.add(const DisclosureSessionStarted('')),
    expect: () => [isA<DisclosureCheckUrl>()],
  );

  blocTest(
    'when startDisclosure returns StartDisclosureReadyToDisclose for login type disclosure, the bloc emits DisclosureCheckOrganizationForLogin',
    setUp: () {
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureReadyToDisclose(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'requestPurpose'.untranslated,
            sessionType: DisclosureSessionType.sameDevice,
            type: DisclosureType.login,
            policy: WalletMockData.policy,
            sharedDataWithOrganizationBefore: false,
            cardRequests: [],
          ),
        );
      });
    },
    build: create,
    act: (bloc) => bloc.add(const DisclosureSessionStarted('')),
    expect: () => [isA<DisclosureCheckOrganizationForLogin>()],
  );

  blocTest(
    'when startDisclosure returns StartDisclosureMissingAttributes, the bloc emits DisclosureMissingAttributes',
    setUp: () {
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureMissingAttributes(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'requestPurpose'.untranslated,
            sessionType: DisclosureSessionType.crossDevice,
            sharedDataWithOrganizationBefore: false,
            missingAttributes: [],
          ),
        );
      });
    },
    build: create,
    act: (bloc) => bloc.add(const DisclosureSessionStarted('')),
    expect: () => [isA<DisclosureMissingAttributes>()],
  );

  blocTest(
    'when the user stops disclosure while checking the organization for ready to disclose, the bloc emits DisclosureStopped and cancels disclosure',
    setUp: () {
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureReadyToDisclose(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'requestPurpose'.untranslated,
            sessionType: DisclosureSessionType.crossDevice,
            type: DisclosureType.regular,
            cardRequests: [],
            policy: WalletMockData.policy,
            sharedDataWithOrganizationBefore: false,
          ),
        );
      });
    },
    build: create,
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      // Give the bloc 25ms to process the previous event
      await Future.delayed(const Duration(milliseconds: 25));
      bloc.add(const DisclosureStopRequested());
    },
    expect: () => [isA<DisclosureCheckUrl>(), isA<DisclosureLoadInProgress>(), isA<DisclosureStopped>()],
    verify: (bloc) => verify(cancelDisclosureUseCase.invoke()).called(greaterThan(0)),
  );

  blocTest(
    'when the user stops disclosure while checking the organization for missing attributes, the bloc emits DisclosureStopped and cancels disclosure',
    setUp: () {
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureMissingAttributes(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'requestPurpose'.untranslated,
            sessionType: DisclosureSessionType.crossDevice,
            missingAttributes: [],
            sharedDataWithOrganizationBefore: false,
          ),
        );
      });
    },
    build: create,
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      // Give the bloc 25ms to process the previous event
      await Future.delayed(const Duration(milliseconds: 25));
      bloc.add(const DisclosureStopRequested());
    },
    expect: () => [
      isA<DisclosureMissingAttributes>(),
      isA<DisclosureLoadInProgress>(),
      isA<DisclosureStopped>(),
    ],
    verify: (bloc) => verify(cancelDisclosureUseCase.invoke()).called(greaterThan(0)),
  );

  blocTest(
    'when the user continues regular disclosure after checking the organization based on StartDisclosureReadyToDisclose, the bloc emits DisclosureConfirmDataAttributes',
    setUp: () {
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureReadyToDisclose(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'requestPurpose'.untranslated,
            sessionType: DisclosureSessionType.crossDevice,
            type: DisclosureType.regular,
            cardRequests: [],
            policy: WalletMockData.policy,
            sharedDataWithOrganizationBefore: false,
          ),
        );
      });
    },
    build: create,
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      // Give the bloc 25ms to process the previous event
      await Future.delayed(const Duration(milliseconds: 25));
      bloc.add(const DisclosureUrlApproved());
    },
    expect: () => [isA<DisclosureCheckUrl>(), isA<DisclosureConfirmDataAttributes>()],
  );

  blocTest(
    'when the user continues login type disclosure after checking the organization based on StartDisclosureReadyToDisclose, the bloc emits DisclosureConfirmPin',
    setUp: () {
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureReadyToDisclose(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'requestPurpose'.untranslated,
            sessionType: DisclosureSessionType.sameDevice,
            type: DisclosureType.login,
            cardRequests: [],
            policy: WalletMockData.policy,
            sharedDataWithOrganizationBefore: false,
          ),
        );
      });
    },
    build: create,
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      // Give the bloc 25ms to process the previous event
      await Future.delayed(const Duration(milliseconds: 25));
      bloc.add(const DisclosureShareRequestedCardsApproved());
    },
    expect: () => [
      isA<DisclosureCheckOrganizationForLogin>(),
      isA<DisclosureConfirmPin>(),
    ],
  );

  blocTest(
    'when the user opts to share the requested attributes, the bloc emits DisclosureConfirmPin',
    setUp: () {
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureReadyToDisclose(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'requestPurpose'.untranslated,
            sessionType: DisclosureSessionType.crossDevice,
            type: DisclosureType.regular,
            cardRequests: [],
            policy: WalletMockData.policy,
            sharedDataWithOrganizationBefore: false,
          ),
        );
      });
    },
    build: create,
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      // Give the bloc 25ms to process the previous event
      await Future.delayed(const Duration(milliseconds: 25));
      bloc.add(const DisclosureUrlApproved());
      bloc.add(const DisclosureShareRequestedCardsApproved());
    },
    skip: 2,
    expect: () => [isA<DisclosureConfirmPin>()],
  );

  blocTest(
    'when the user confirms the pin, the bloc emits DisclosureSuccess',
    setUp: () {
      when(getMostRecentWalletEventUsecase.invoke()).thenAnswer((_) async => WalletMockData.disclosureEvent);
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureReadyToDisclose(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'requestPurpose'.untranslated,
            sessionType: DisclosureSessionType.crossDevice,
            type: DisclosureType.regular,
            cardRequests: [],
            policy: WalletMockData.policy,
            sharedDataWithOrganizationBefore: false,
          ),
        );
      });
    },
    build: create,
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      // Give the bloc 25ms to process the previous event
      await Future.delayed(const Duration(milliseconds: 25));
      bloc.add(const DisclosureUrlApproved());
      bloc.add(const DisclosureShareRequestedCardsApproved());
      bloc.add(const DisclosurePinConfirmed());
    },
    skip: 3,
    expect: () => [isA<DisclosureSuccess>()],
  );

  blocTest(
    'when the user leaves feedback when stopping, the bloc emits DisclosureLeftFeedback and disclosure is cancelled',
    setUp: () {
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureReadyToDisclose(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'requestPurpose'.untranslated,
            sessionType: DisclosureSessionType.crossDevice,
            type: DisclosureType.regular,
            cardRequests: [],
            policy: WalletMockData.policy,
            sharedDataWithOrganizationBefore: false,
          ),
        );
      });
    },
    build: create,
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      // Give the bloc 25ms to process the previous event
      await Future.delayed(const Duration(milliseconds: 25));
      bloc.add(const DisclosureReportPressed(option: ReportingOption.impersonatingOrganization));
    },
    verify: (bloc) => verify(cancelDisclosureUseCase.invoke()).called(greaterThan(0)),
    expect: () => [
      isA<DisclosureCheckUrl>(),
      isA<DisclosureLoadInProgress>(),
      isA<DisclosureLeftFeedback>(),
    ],
  );

  blocTest(
    'when user presses back from the DisclosureConfirmDataAttributes state, the bloc emits DisclosureCheckOrganization ',
    setUp: () {
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureReadyToDisclose(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'requestPurpose'.untranslated,
            sessionType: DisclosureSessionType.crossDevice,
            type: DisclosureType.regular,
            cardRequests: [],
            policy: WalletMockData.policy,
            sharedDataWithOrganizationBefore: false,
          ),
        );
      });
    },
    build: create,
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      // Give the bloc 25ms to process the previous event
      await Future.delayed(const Duration(milliseconds: 25));
      bloc.add(const DisclosureUrlApproved());
      bloc.add(const DisclosureBackPressed());
    },
    expect: () => [
      isA<DisclosureCheckUrl>(),
      isA<DisclosureConfirmDataAttributes>(),
      isA<DisclosureCheckUrl>(),
    ],
  );

  blocTest(
    'when a network error occurs while the user confirms the pin, the bloc emits DisclosureNetworkError',
    setUp: () {
      when(CoreErrorExtension.networkRepository.hasInternet()).thenAnswer((realInvocation) async {
        await Future.delayed(const Duration(milliseconds: 100));
        return false;
      });
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureReadyToDisclose(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'requestPurpose'.untranslated,
            sessionType: DisclosureSessionType.crossDevice,
            type: DisclosureType.regular,
            cardRequests: [],
            policy: WalletMockData.policy,
            sharedDataWithOrganizationBefore: false,
          ),
        );
      });
    },
    build: create,
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      // Give the bloc 25ms to process the previous event
      await Future.delayed(const Duration(milliseconds: 25));
      bloc.add(const DisclosureUrlApproved());
      bloc.add(const DisclosureShareRequestedCardsApproved());
      bloc.add(const DisclosureConfirmPinFailed(error: NetworkError(hasInternet: false, sourceError: 'test')));
    },
    wait: const Duration(milliseconds: 150),
    skip: 4,
    expect: () => [isA<DisclosureNetworkError>()],
  );

  blocTest(
    'when user presses back from the DisclosureConfirmPin state, the bloc emits DisclosureConfirmDataAttributes ',
    setUp: () {
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureReadyToDisclose(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'requestPurpose'.untranslated,
            sessionType: DisclosureSessionType.crossDevice,
            type: DisclosureType.regular,
            cardRequests: [],
            policy: WalletMockData.policy,
            sharedDataWithOrganizationBefore: false,
          ),
        );
      });
    },
    build: create,
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      // Give the bloc 25ms to process the previous event
      await Future.delayed(const Duration(milliseconds: 25));
      bloc.add(const DisclosureUrlApproved());
      bloc.add(const DisclosureShareRequestedCardsApproved());
      bloc.add(const DisclosureBackPressed());
    },
    skip: 1,
    expect: () => [
      isA<DisclosureConfirmDataAttributes>(),
      isA<DisclosureConfirmPin>(),
      isA<DisclosureConfirmDataAttributes>(),
    ],
  );

  blocTest(
    'when user presses back from the DisclosureConfirmPin state for login type disclosure, the bloc emits DisclosureCheckOrganizationForLogin',
    setUp: () {
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureReadyToDisclose(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'requestPurpose'.untranslated,
            sessionType: DisclosureSessionType.sameDevice,
            type: DisclosureType.login,
            cardRequests: [],
            policy: WalletMockData.policy,
            sharedDataWithOrganizationBefore: false,
          ),
        );
      });
    },
    build: create,
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      // Give the bloc 25ms to process the previous event
      await Future.delayed(const Duration(milliseconds: 25));
      bloc.add(const DisclosureShareRequestedCardsApproved());
      bloc.add(const DisclosureBackPressed());
    },
    skip: 1,
    expect: () => [
      isA<DisclosureConfirmPin>(),
      isA<DisclosureCheckOrganizationForLogin>(),
    ],
  );

  blocTest(
    'startDisclosure is called with isQrCode set to true',
    setUp: () => when(
      startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode')),
    ).thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test'))),
    build: create,
    act: (bloc) => bloc.add(const DisclosureSessionStarted('http://true', isQrCode: true)),
    verify: (_) => verify(startDisclosureUseCase.invoke('http://true', isQrCode: true)).called(1),
  );

  blocTest(
    'startDisclosure is called with isQrCode set to false',
    setUp: () => when(
      startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode')),
    ).thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test'))),
    build: create,
    act: (bloc) => bloc.add(const DisclosureSessionStarted('http://false', isQrCode: false)),
    verify: (_) => verify(startDisclosureUseCase.invoke('http://false', isQrCode: false)).called(1),
  );

  blocTest(
    'when a CoreDisclosureSourceMismatchError(isCrossDevice=true) is thrown, emit the DisclosureExternalScannerError',
    setUp: () => when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer(
      (_) async => const Result.error(
        ExternalScannerError(
          sourceError: CoreDisclosureSourceMismatchError('description', isCrossDevice: true),
        ),
      ),
    ),
    build: create,
    act: (bloc) async => bloc.add(const DisclosureSessionStarted('')),
    expect: () => [
      isA<DisclosureLoadInProgress>(),
      isA<DisclosureExternalScannerError>(),
    ],
  );

  blocTest(
    'when a CoreExpiredSessionError is thrown when starting disclosure, emit DisclosureSessionExpired',
    setUp: () => when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer(
      (_) async => const Result.error(
        SessionError(state: SessionState.expired, canRetry: true, sourceError: 'test'),
      ),
    ),
    build: create,
    act: (bloc) async => bloc.add(const DisclosureSessionStarted('')),
    expect: () => [
      isA<DisclosureLoadInProgress>(),
      isA<DisclosureSessionExpired>()
          .having((error) => error.canRetry, 'canRetry', true)
          .having((error) => error.isCrossDevice, 'isCrossDevice', false),
    ],
  );

  blocTest(
    'when a CoreExpiredSessionError is thrown when accepting disclosure, emit DisclosureSessionExpired',
    setUp: () => when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer(
      (_) async => Result.success(
        StartDisclosureReadyToDisclose(
          relyingParty: WalletMockData.organization,
          originUrl: 'originUrl',
          requestPurpose: ''.untranslated,
          sessionType: DisclosureSessionType.crossDevice,
          type: DisclosureType.regular,
          cardRequests: [],
          policy: WalletMockData.policy,
          sharedDataWithOrganizationBefore: false,
        ),
      ),
    ),
    build: create,
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      await Future.delayed(const Duration(milliseconds: 20));
      bloc.add(const DisclosureUrlApproved());
      await Future.delayed(const Duration(milliseconds: 20));
      bloc.add(
        const DisclosureConfirmPinFailed(
          error: SessionError(
            state: SessionState.expired,
            canRetry: false,
            crossDevice: SessionType.sameDevice,
            sourceError: 'test',
          ),
        ),
      );
    },
    expect: () => [
      isA<DisclosureCheckUrl>(),
      isA<DisclosureConfirmDataAttributes>(),
      isA<DisclosureLoadInProgress>(),
      const DisclosureSessionExpired(
        canRetry: false,
        isCrossDevice: true,
        error: SessionError(
          state: SessionState.expired,
          canRetry: false,
          crossDevice: SessionType.sameDevice,
          sourceError: 'test',
        ),
      ),
    ],
  );

  blocTest(
    'when a CoreGenericError with a returnUrl is thrown, the bloc emits a GenericError that contains this returnUrl',
    setUp: () {
      return when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer(
        (_) async => const Result.error(
          GenericError('', redirectUrl: 'https://example.org', sourceError: 'test'),
        ),
      );
    },
    build: create,
    act: (bloc) async => bloc.add(const DisclosureSessionStarted('')),
    expect: () => [
      isA<DisclosureLoadInProgress>(),
      isA<DisclosureGenericError>().having(
        (error) => error.returnUrl,
        'return url matches that of the error',
        'https://example.org',
      ),
    ],
  );

  blocTest(
    'when disclosure is stopped and a returnUrl is provided, this returnUrl is available inside the stopped state',
    setUp: () {
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureReadyToDisclose(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'requestPurpose'.untranslated,
            sessionType: DisclosureSessionType.sameDevice,
            type: DisclosureType.login,
            cardRequests: [],
            policy: WalletMockData.policy,
            sharedDataWithOrganizationBefore: false,
          ),
        );
      });
      when(cancelDisclosureUseCase.invoke()).thenAnswer((_) async => const Result.success('http://example.org'));
    },
    build: create,
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      await Future.delayed(const Duration(milliseconds: 25));
      bloc.add(const DisclosureStopRequested());
    },
    expect: () => [
      isA<DisclosureCheckOrganizationForLogin>(),
      isA<DisclosureLoadInProgress>(),
      DisclosureStopped(
        organization: WalletMockData.organization,
        isLoginFlow: true,
        returnUrl: 'http://example.org',
      ),
    ],
  );

  blocTest(
    'when a CoreSessionCancelledError is thrown when accepting disclosure, emit DisclosureCancelledSessionError',
    setUp: () => when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer(
      (_) async => Result.success(
        StartDisclosureReadyToDisclose(
          relyingParty: WalletMockData.organization,
          originUrl: 'originUrl',
          requestPurpose: ''.untranslated,
          sessionType: DisclosureSessionType.crossDevice,
          type: DisclosureType.regular,
          cardRequests: [],
          policy: WalletMockData.policy,
          sharedDataWithOrganizationBefore: false,
        ),
      ),
    ),
    build: create,
    act: (bloc) async {
      bloc.add(const DisclosureSessionStarted(''));
      await Future.delayed(const Duration(milliseconds: 20));
      bloc.add(const DisclosureUrlApproved());
      await Future.delayed(const Duration(milliseconds: 20));
      bloc.add(
        const DisclosureConfirmPinFailed(
          error: SessionError(
            state: SessionState.cancelled,
            crossDevice: SessionType.crossDevice,
            sourceError: 'test',
          ),
        ),
      );
    },
    expect: () => [
      isA<DisclosureCheckUrl>(),
      isA<DisclosureConfirmDataAttributes>(),
      isA<DisclosureLoadInProgress>(),
      DisclosureSessionCancelled(
        error: const SessionError(
          state: SessionState.cancelled,
          crossDevice: SessionType.crossDevice,
          sourceError: 'test',
        ),
        relyingParty: WalletMockData.organization,
      ),
    ],
  );

  blocTest(
    'card selection state is maintained when navigating back and forth between CheckUrl and ConfirmDataAttributes',
    setUp: () {
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureReadyToDisclose(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'testPurpose'.untranslated,
            sessionType: DisclosureSessionType.crossDevice,
            type: DisclosureType.regular,
            policy: WalletMockData.policy,
            sharedDataWithOrganizationBefore: false,
            cardRequests: [
              WalletMockData.discloseCardRequestSingleCard,
              WalletMockData.discloseCardRequestMultiCard,
            ],
          ),
        );
      });
    },
    build: create,
    act: (bloc) async {
      // 1. Start session → CheckUrl
      bloc.add(const DisclosureSessionStarted(''));
      await Future.delayed(const Duration(milliseconds: 25));

      // 2. Approve URL → ConfirmDataAttributes (initial state with default selection)
      bloc.add(const DisclosureUrlApproved());
      await Future.delayed(const Duration(milliseconds: 25));

      // 3. Update selection of "disclosureCardRequestMultiCard"
      expect(
        WalletMockData.discloseCardRequestMultiCard.hasAlternatives,
        isTrue,
        reason: 'Sanity check to verify we are working with compatible request',
      );
      bloc.add(
        DisclosureAlternativeCardSelected(
          WalletMockData.discloseCardRequestMultiCard.select(
            WalletMockData.discloseCardRequestMultiCard.alternatives.first,
          ),
        ),
      );
      await Future.delayed(const Duration(milliseconds: 25));

      // 4. Press back → CheckUrl
      bloc.add(const DisclosureBackPressed());
      await Future.delayed(const Duration(milliseconds: 25));

      // 5. Approve URL again → ConfirmDataAttributes (should preserve selection)
      bloc.add(const DisclosureUrlApproved());
    },
    expect: () {
      return [
        isA<DisclosureCheckUrl>(),
        isA<DisclosureConfirmDataAttributes>().having(
          (state) => state.cardRequests.map((it) => it.selectedIndex),
          'initial selection is all 0',
          [0, 0],
        ),
        isA<DisclosureConfirmDataAttributes>().having(
          (state) => state.cardRequests.map((it) => it.selectedIndex),
          'update selection of second request',
          [0, 1],
        ),
        isA<DisclosureCheckUrl>(),
        isA<DisclosureConfirmDataAttributes>().having(
          (state) => state.cardRequests.map((it) => it.selectedIndex),
          'updated selection should be maintained',
          [0, 1],
        ),
      ];
    },
  );

  blocTest(
    'card selection state is maintained when navigating back from ConfirmPin to ConfirmDataAttributes',
    setUp: () {
      when(startDisclosureUseCase.invoke(any, isQrCode: anyNamed('isQrCode'))).thenAnswer((_) async {
        return Result.success(
          StartDisclosureReadyToDisclose(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            requestPurpose: 'testPurpose'.untranslated,
            sessionType: DisclosureSessionType.crossDevice,
            type: DisclosureType.regular,
            policy: WalletMockData.policy,
            sharedDataWithOrganizationBefore: false,
            cardRequests: [
              WalletMockData.discloseCardRequestSingleCard,
              WalletMockData.discloseCardRequestMultiCard,
            ],
          ),
        );
      });
    },
    build: create,
    act: (bloc) async {
      // 1. Start session → CheckUrl
      bloc.add(const DisclosureSessionStarted(''));
      await Future.delayed(const Duration(milliseconds: 25));

      // 2. Approve URL → ConfirmDataAttributes (initial state with default selection)
      bloc.add(const DisclosureUrlApproved());
      await Future.delayed(const Duration(milliseconds: 25));

      // 3. Update selection of "disclosureCardRequestMultiCard"
      expect(
        WalletMockData.discloseCardRequestMultiCard.hasAlternatives,
        isTrue,
        reason: 'Sanity check to verify we are working with compatible request',
      );
      bloc.add(
        DisclosureAlternativeCardSelected(
          WalletMockData.discloseCardRequestMultiCard.select(
            WalletMockData.discloseCardRequestMultiCard.alternatives.first,
          ),
        ),
      );
      await Future.delayed(const Duration(milliseconds: 25));

      // 4. Proceed to ConfirmPin
      bloc.add(const DisclosureShareRequestedCardsApproved());
      await Future.delayed(const Duration(milliseconds: 25));

      // 5. Press back → ConfirmDataAttributes
      bloc.add(const DisclosureBackPressed());
      await Future.delayed(const Duration(milliseconds: 25));
    },
    expect: () {
      return [
        isA<DisclosureCheckUrl>(),
        isA<DisclosureConfirmDataAttributes>().having(
          (state) => state.cardRequests.map((it) => it.selectedIndex),
          'initial selection is all 0',
          [0, 0],
        ),
        isA<DisclosureConfirmDataAttributes>().having(
          (state) => state.cardRequests.map((it) => it.selectedIndex),
          'update selection of second request',
          [0, 1],
        ),
        isA<DisclosureConfirmPin>(),
        isA<DisclosureConfirmDataAttributes>().having(
          (state) => state.cardRequests.map((it) => it.selectedIndex),
          'updated selection should be maintained after returning from ConfirmPin',
          [0, 1],
        ),
      ];
    },
  );
}
