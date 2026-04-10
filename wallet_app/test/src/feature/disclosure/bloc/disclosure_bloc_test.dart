import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:mockito/mockito.dart';
import 'package:test/test.dart';
import 'package:wallet/src/data/repository/disclosure/disclosure_repository.dart';
import 'package:wallet/src/domain/model/close_proximity/ble_connection_event.dart';
import 'package:wallet/src/domain/model/disclosure/disclose_card_request.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_session_type.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_type.dart';
import 'package:wallet/src/domain/model/disclosure/start_disclosure_request.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/feature/disclosure/bloc/disclosure_bloc.dart';
import 'package:wallet/src/feature/report_issue/reporting_option.dart';
import 'package:wallet/src/util/extension/core_error_extension.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart' hide SessionType;

import '../../../mocks/wallet_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockStartDisclosureUseCase startDisclosureUseCase;
  late MockCancelDisclosureUseCase cancelDisclosureUseCase;
  late MockGetMostRecentWalletEventUseCase getMostRecentWalletEventUseCase;
  late MockObserveCloseProximityConnectionUseCase observeCloseProximityConnectionUseCase;

  /// Create a new [DisclosureBloc] configured with the (mocked) usecases
  DisclosureBloc create() => DisclosureBloc(
    startDisclosureUseCase,
    cancelDisclosureUseCase,
    getMostRecentWalletEventUseCase,
    observeCloseProximityConnectionUseCase,
  );

  setUp(() {
    startDisclosureUseCase = MockStartDisclosureUseCase();
    cancelDisclosureUseCase = MockCancelDisclosureUseCase();
    getMostRecentWalletEventUseCase = MockGetMostRecentWalletEventUseCase();
    observeCloseProximityConnectionUseCase = MockObserveCloseProximityConnectionUseCase();
    when(observeCloseProximityConnectionUseCase.invoke()).thenAnswer((_) => const Stream.empty());
  });

  test('initial state is correct', () {
    expect(create().state, const DisclosureInitial());
  });

  group('Start Disclosure', () {
    group('Same device', () {
      blocTest(
        'when startDisclosure returns StartDisclosureReadyToDisclose for regular disclosure, the bloc emits DisclosureConfirmDataAttributes',
        setUp: () {
          when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
            return Result.success(emptyRequest(sessionType: .sameDevice, type: .regular));
          });
        },
        build: create,
        act: (bloc) => bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink(''))),
        expect: () => [isA<DisclosureConfirmDataAttributes>()],
      );

      blocTest(
        'when startDisclosure returns StartDisclosureReadyToDisclose for login type disclosure, the bloc emits DisclosureCheckOrganizationForLogin',
        setUp: () {
          when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
            return Result.success(emptyRequest(sessionType: .sameDevice, type: .login));
          });
        },
        build: create,
        act: (bloc) => bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink(''))),
        expect: () => [isA<DisclosureCheckOrganizationForLogin>()],
      );
    });

    group('Cross device', () {
      blocTest(
        'when startDisclosure returns StartDisclosureReadyToDisclose for regular disclosure, the bloc emits DisclosureCheckUrl',
        setUp: () {
          when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
            return Result.success(emptyRequest(sessionType: .crossDevice, type: .regular));
          });
        },
        build: create,
        act: (bloc) => bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.qrScan(''))),
        expect: () => [isA<DisclosureCheckUrl>()],
      );

      blocTest(
        'when startDisclosure returns StartDisclosureReadyToDisclose for login type disclosure, the bloc emits DisclosureCheckUrl',
        setUp: () {
          when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
            return Result.success(emptyRequest(sessionType: .crossDevice, type: .login));
          });
        },
        build: create,
        act: (bloc) => bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.qrScan(''))),
        expect: () => [isA<DisclosureCheckUrl>()],
      );
    });

    blocTest(
      'when startDisclosure returns StartDisclosureMissingAttributes, the bloc emits DisclosureMissingAttributes',
      setUp: () {
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
          return Result.success(missingAttributesRequest(sessionType: .crossDevice));
        });
      },
      build: create,
      act: (bloc) => bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink(''))),
      expect: () => [isA<DisclosureMissingAttributes>()],
    );

    blocTest(
      'startDisclosure is called with isQrCode set to true',
      setUp: () => when(
        startDisclosureUseCase.invoke(any),
      ).thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test'))),
      build: create,
      act: (bloc) => bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.qrScan('http://true'))),
      verify: (_) =>
          verify(startDisclosureUseCase.invoke(const StartDisclosureRequest.qrScan('http://true'))).called(1),
    );

    blocTest(
      'startDisclosure is called with isQrCode set to false',
      setUp: () => when(
        startDisclosureUseCase.invoke(any),
      ).thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test'))),
      build: create,
      act: (bloc) => bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink('http://false'))),
      verify: (_) =>
          verify(startDisclosureUseCase.invoke(const StartDisclosureRequest.deeplink('http://false'))).called(1),
    );
  });

  group('Navigation & Flow', () {
    blocTest(
      'when the user continues regular disclosure after checking the organization based on StartDisclosureReadyToDisclose, the bloc emits DisclosureConfirmDataAttributes',
      setUp: () {
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
          return Result.success(emptyRequest(sessionType: .crossDevice, type: .regular));
        });
      },
      build: create,
      act: (bloc) async {
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink('')));
        // Give the bloc 25ms to process the previous event
        await Future.delayed(const Duration(milliseconds: 25));
        bloc.add(const DisclosureUrlApproved());
      },
      expect: () => [isA<DisclosureCheckUrl>(), isA<DisclosureConfirmDataAttributes>()],
    );

    blocTest(
      'when the user continues login type disclosure after checking the organization based on StartDisclosureReadyToDisclose, the bloc emits DisclosureConfirmPin',
      setUp: () {
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
          return Result.success(emptyRequest(sessionType: .sameDevice, type: .login));
        });
      },
      build: create,
      act: (bloc) async {
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink('')));
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
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
          return Result.success(emptyRequest(sessionType: .crossDevice, type: .regular));
        });
      },
      build: create,
      act: (bloc) async {
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.qrScan('')));
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
        when(getMostRecentWalletEventUseCase.invoke()).thenAnswer((_) async => WalletMockData.disclosureEvent);
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
          return Result.success(emptyRequest(sessionType: .crossDevice, type: .regular));
        });
      },
      build: create,
      act: (bloc) async {
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.qrScan('')));
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
      'when user presses back from the DisclosureConfirmDataAttributes state, the bloc emits DisclosureCheckUrl ',
      setUp: () {
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
          return Result.success(emptyRequest(sessionType: .crossDevice, type: .regular));
        });
      },
      build: create,
      act: (bloc) async {
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.qrScan('')));
        // Give the bloc 25ms to process the previous event
        await Future.delayed(const Duration(milliseconds: 25));
        bloc.add(const DisclosureUrlApproved());
        bloc.add(const DisclosureBackPressed());
      },
      expect: () => [
        isA<DisclosureCheckUrl>(),
        isA<DisclosureConfirmDataAttributes>(),
        isA<DisclosureCheckUrl>().having(
          (it) => it.afterBackPressed,
          'backPressed flag should be true after navigating back',
          isTrue,
        ),
      ],
    );

    blocTest(
      'when user presses back from the DisclosureConfirmPin state, the bloc emits DisclosureConfirmDataAttributes ',
      setUp: () {
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
          return Result.success(emptyRequest(sessionType: .crossDevice, type: .regular));
        });
      },
      build: create,
      act: (bloc) async {
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.qrScan('')));
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
        isA<DisclosureConfirmDataAttributes>().having(
          (it) => it.afterBackPressed,
          'backPressed flag should be true after navigating back',
          isTrue,
        ),
      ],
    );

    blocTest(
      'when user presses back from the DisclosureConfirmPin state for login type disclosure, the bloc emits DisclosureCheckOrganizationForLogin',
      setUp: () {
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
          return Result.success(emptyRequest(sessionType: .sameDevice, type: .login));
        });
      },
      build: create,
      act: (bloc) async {
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink('')));
        // Give the bloc 25ms to process the previous event
        await Future.delayed(const Duration(milliseconds: 25));
        bloc.add(const DisclosureShareRequestedCardsApproved());
        bloc.add(const DisclosureBackPressed());
      },
      expect: () => [
        isA<DisclosureCheckOrganizationForLogin>(),
        isA<DisclosureConfirmPin>(),
        isA<DisclosureCheckOrganizationForLogin>().having(
          (it) => it.afterBackPressed,
          'backPressed flag should be true after navigating back',
          isTrue,
        ),
      ],
    );
  });

  group('Card Selection', () {
    blocTest(
      'card selection state is maintained when navigating back and forth between CheckUrl and ConfirmDataAttributes',
      setUp: () {
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
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
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.qrScan('')));
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
            (state) => state.cardRequests.selectedIndices,
            'initial selection is all 0',
            [0, 0],
          ),
          isA<DisclosureConfirmDataAttributes>().having(
            (state) => state.cardRequests.selectedIndices,
            'update selection of second request',
            [0, 1],
          ),
          isA<DisclosureCheckUrl>(),
          isA<DisclosureConfirmDataAttributes>().having(
            (state) => state.cardRequests.selectedIndices,
            'updated selection should be maintained',
            [0, 1],
          ),
        ];
      },
    );

    blocTest(
      'card selection state is maintained when navigating back from ConfirmPin to ConfirmDataAttributes',
      setUp: () {
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
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
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink('')));
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
            (state) => state.cardRequests.selectedIndices,
            'initial selection is all 0',
            [0, 0],
          ),
          isA<DisclosureConfirmDataAttributes>().having(
            (state) => state.cardRequests.selectedIndices,
            'update selection of second request',
            [0, 1],
          ),
          isA<DisclosureConfirmPin>(),
          isA<DisclosureConfirmDataAttributes>().having(
            (state) => state.cardRequests.selectedIndices,
            'updated selection should be maintained after returning from ConfirmPin',
            [0, 1],
          ),
        ];
      },
    );
  });

  group('Stopping & Cancelling', () {
    blocTest(
      'when the user stops disclosure while checking the organization for ready to disclose, the bloc emits DisclosureStopped and cancels disclosure',
      setUp: () {
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
          return Result.success(emptyRequest(sessionType: .crossDevice, type: .regular));
        });
      },
      build: create,
      act: (bloc) async {
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.qrScan('')));
        // Give the bloc 25ms to process the previous event
        await Future.delayed(const Duration(milliseconds: 25));
        bloc.add(const DisclosureStopRequested());
      },
      expect: () => [isA<DisclosureCheckUrl>(), isA<DisclosureLoadInProgress>(), isA<DisclosureStopped>()],
      verify: (bloc) => verify(cancelDisclosureUseCase.invoke()).called(greaterThan(0)),
    );

    blocTest(
      'when BLoC receives DisclosureCancelRequested event, the active session is cancelled',
      build: create,
      act: (bloc) => bloc.add(const DisclosureCancelRequested()),
      verify: (bloc) => verify(cancelDisclosureUseCase.invoke()).called(1),
    );

    blocTest(
      'when the user stops disclosure while checking the organization for missing attributes, the bloc emits DisclosureStopped and cancels disclosure',
      setUp: () {
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
          return Result.success(missingAttributesRequest(sessionType: .crossDevice));
        });
      },
      build: create,
      act: (bloc) async {
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.qrScan('')));
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
      'when the user leaves feedback when stopping, the bloc emits DisclosureLeftFeedback and disclosure is cancelled',
      setUp: () {
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
          return Result.success(emptyRequest(sessionType: .crossDevice, type: .regular));
        });
      },
      build: create,
      act: (bloc) async {
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.qrScan('')));
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
      'when disclosure is stopped and a returnUrl is provided, this returnUrl is available inside the stopped state',
      setUp: () {
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
          return Result.success(emptyRequest(sessionType: .sameDevice, type: .login));
        });
        when(cancelDisclosureUseCase.invoke()).thenAnswer((_) async => const Result.success('http://example.org'));
      },
      build: create,
      act: (bloc) async {
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink('')));
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
  });

  group('Error Handling', () {
    blocTest(
      'when startDisclosure fails, emit generic error',
      setUp: () => when(
        startDisclosureUseCase.invoke(any),
      ).thenAnswer((_) async => const Result.error(GenericError('', sourceError: 'test'))),
      build: create,
      act: (bloc) => bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink(''))),
      expect: () => [
        isA<DisclosureLoadInProgress>(),
        isA<DisclosureGenericError>(),
      ],
    );

    blocTest(
      'when startDisclosure fails with network issue, emit DisclosureNetworkError(hasInternet: true)',
      setUp: () => when(
        startDisclosureUseCase.invoke(any),
      ).thenAnswer((_) async => const Result.error(NetworkError(hasInternet: true, sourceError: 'test'))),
      build: create,
      act: (bloc) => bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink(''))),
      verify: (bloc) {
        expect(bloc.state, isA<DisclosureNetworkError>());
        expect((bloc.state as DisclosureNetworkError).error.hasInternet, isTrue);
        expect((bloc.state as DisclosureNetworkError).error, isA<NetworkError>());
      },
    );

    blocTest(
      'when startDisclosure fails with network issue and there is no internet, emit DisclosureNetworkError(hasInternet: false)',
      setUp: () {
        when(
          startDisclosureUseCase.invoke(any),
        ).thenAnswer((_) async => const Result.error(NetworkError(hasInternet: false, sourceError: 'test')));
        when(CoreErrorExtension.networkRepository.hasInternet()).thenAnswer((realInvocation) async {
          await Future.delayed(const Duration(milliseconds: 100));
          return false;
        });
      },
      wait: const Duration(milliseconds: 150),
      build: create,
      act: (bloc) => bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink(''))),
      verify: (bloc) {
        expect(bloc.state, isA<DisclosureNetworkError>());
        expect((bloc.state as DisclosureNetworkError).error.hasInternet, isFalse);
        expect((bloc.state as DisclosureNetworkError).error, isA<NetworkError>());
      },
    );

    blocTest(
      'when a network error occurs while the user confirms the pin, the bloc emits DisclosureNetworkError',
      setUp: () {
        when(CoreErrorExtension.networkRepository.hasInternet()).thenAnswer((realInvocation) async {
          await Future.delayed(const Duration(milliseconds: 100));
          return false;
        });
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
          return Result.success(emptyRequest(sessionType: .crossDevice, type: .regular));
        });
      },
      build: create,
      act: (bloc) async {
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink('')));
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
      'when a CoreDisclosureSourceMismatchError(isCrossDevice=true) is thrown, emit the DisclosureExternalScannerError',
      setUp: () => when(startDisclosureUseCase.invoke(any)).thenAnswer(
        (_) async => const Result.error(
          ExternalScannerError(
            sourceError: CoreDisclosureSourceMismatchError('description', isCrossDevice: true),
          ),
        ),
      ),
      build: create,
      act: (bloc) async => bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink(''))),
      expect: () => [
        isA<DisclosureLoadInProgress>(),
        isA<DisclosureExternalScannerError>(),
      ],
    );

    blocTest(
      'when a CoreExpiredSessionError is thrown when starting disclosure, emit DisclosureSessionExpired',
      setUp: () => when(startDisclosureUseCase.invoke(any)).thenAnswer(
        (_) async => const Result.error(
          SessionError(state: SessionState.expired, canRetry: true, sourceError: 'test'),
        ),
      ),
      build: create,
      act: (bloc) async => bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink(''))),
      expect: () => [
        isA<DisclosureLoadInProgress>(),
        isA<DisclosureSessionExpired>()
            .having((error) => error.canRetry, 'canRetry', true)
            .having((error) => error.isCrossDevice, 'isCrossDevice', false),
      ],
    );

    blocTest(
      'when a CoreExpiredSessionError is thrown when accepting disclosure, emit DisclosureSessionExpired',
      setUp: () => when(startDisclosureUseCase.invoke(any)).thenAnswer(
        (_) async => Result.success(emptyRequest(sessionType: .crossDevice, type: .regular)),
      ),
      build: create,
      act: (bloc) async {
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink('')));
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
        return when(startDisclosureUseCase.invoke(any)).thenAnswer(
          (_) async => const Result.error(
            GenericError('', redirectUrl: 'https://example.org', sourceError: 'test'),
          ),
        );
      },
      build: create,
      act: (bloc) async => bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink(''))),
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
      'when a CoreSessionCancelledError is thrown when accepting disclosure, emit DisclosureCancelledSessionError',
      setUp: () => when(startDisclosureUseCase.invoke(any)).thenAnswer(
        (_) async => Result.success(emptyRequest(sessionType: .crossDevice, type: .regular)),
      ),
      build: create,
      act: (bloc) async {
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink('')));
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
      'when a RelyingPartyError is thrown, emit DisclosureRelyingPartyError',
      setUp: () => when(startDisclosureUseCase.invoke(any)).thenAnswer(
        (_) async => Result.error(
          RelyingPartyError(
            organizationName: 'organizationName'.untranslated,
            sourceError: 'test',
          ),
        ),
      ),
      build: create,
      act: (bloc) async => bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink(''))),
      expect: () => [
        isA<DisclosureLoadInProgress>(),
        isA<DisclosureRelyingPartyError>(),
      ],
    );
  });

  group('Close Proximity', () {
    late StreamController<BleConnectionEvent> bleEventController;

    setUp(() {
      bleEventController = StreamController<BleConnectionEvent>.broadcast();
      when(observeCloseProximityConnectionUseCase.invoke()).thenAnswer((_) => bleEventController.stream);
    });

    tearDown(() {
      bleEventController.close();
    });

    blocTest(
      'when session started with closeProximity, it observes ble events and emits DisclosureConfirmDataAttributes',
      setUp: () {
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
          return Result.success(
            StartDisclosureReadyToDisclose(
              relyingParty: WalletMockData.organization,
              originUrl: 'http://origin.org',
              requestPurpose: 'requestPurpose'.untranslated,
              sessionType: DisclosureSessionType.closeProximity,
              type: DisclosureType.regular,
              cardRequests: [],
              policy: WalletMockData.policy,
              sharedDataWithOrganizationBefore: false,
            ),
          );
        });
      },
      build: create,
      act: (bloc) => bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink(''))),
      expect: () => [isA<DisclosureConfirmDataAttributes>()],
      verify: (bloc) => verify(observeCloseProximityConnectionUseCase.invoke()).called(1),
    );

    blocTest(
      'when ble disconnected, emits DisclosureCloseProximityDisconnected',
      setUp: () {
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
          return Result.success(
            StartDisclosureReadyToDisclose(
              relyingParty: WalletMockData.organization,
              originUrl: 'http://origin.org',
              requestPurpose: 'requestPurpose'.untranslated,
              sessionType: DisclosureSessionType.closeProximity,
              type: DisclosureType.regular,
              cardRequests: [],
              policy: WalletMockData.policy,
              sharedDataWithOrganizationBefore: false,
            ),
          );
        });
        when(cancelDisclosureUseCase.invoke()).thenAnswer((_) async => const Result.success(''));
      },
      build: create,
      act: (bloc) async {
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink('')));
        await Future.delayed(Duration.zero);
        bleEventController.add(const BleDisconnected());
      },
      expect: () => [
        isA<DisclosureConfirmDataAttributes>(),
        const DisclosureCloseProximityDisconnected(isLoginFlow: false),
      ],
      verify: (bloc) => verify(cancelDisclosureUseCase.invoke()).called(1),
    );

    blocTest(
      'when ble error occurs, handles as application error',
      setUp: () {
        when(startDisclosureUseCase.invoke(any)).thenAnswer((_) async {
          return Result.success(
            StartDisclosureReadyToDisclose(
              relyingParty: WalletMockData.organization,
              originUrl: 'http://origin.org',
              requestPurpose: 'requestPurpose'.untranslated,
              sessionType: DisclosureSessionType.closeProximity,
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
        bloc.add(const DisclosureSessionStarted(StartDisclosureRequest.deeplink('')));
        await Future.delayed(Duration.zero);
        bleEventController.add(const BleError(CoreGenericError('test')));
      },
      expect: () => [
        isA<DisclosureConfirmDataAttributes>(),
        isA<DisclosureLoadInProgress>(),
        isA<DisclosureGenericError>(),
      ],
    );
  });
}

// Helper method, creates an empty StartDisclosureReadyToDisclose event
StartDisclosureReadyToDisclose emptyRequest({
  required DisclosureSessionType sessionType,
  required DisclosureType type,
}) => StartDisclosureReadyToDisclose(
  relyingParty: WalletMockData.organization,
  originUrl: 'http://origin.org',
  requestPurpose: 'requestPurpose'.untranslated,
  sessionType: sessionType,
  type: type,
  policy: WalletMockData.policy,
  sharedDataWithOrganizationBefore: false,
  cardRequests: [],
);

// Helper method, creates an empty StartDisclosureMissingAttributes event
StartDisclosureMissingAttributes missingAttributesRequest({
  required DisclosureSessionType sessionType,
}) => StartDisclosureMissingAttributes(
  relyingParty: WalletMockData.organization,
  originUrl: 'http://origin.org',
  requestPurpose: 'requestPurpose'.untranslated,
  sessionType: sessionType,
  sharedDataWithOrganizationBefore: false,
  missingAttributes: [],
);
