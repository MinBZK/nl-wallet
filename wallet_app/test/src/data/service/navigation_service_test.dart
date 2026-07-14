import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:wallet/src/data/service/navigation_service.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/domain/model/update/update_notification.dart';
import 'package:wallet/src/domain/model/wallet_state.dart';
import 'package:wallet/src/feature/common/dialog/generic_dialog.dart';
import 'package:wallet/src/feature/common/dialog/update_notification_dialog.dart';
import 'package:wallet/src/feature/disclosure/argument/disclosure_screen_argument.dart';
import 'package:wallet/src/feature/issuance/argument/issuance_screen_argument.dart';
import 'package:wallet/src/feature/sign/argument/sign_screen_argument.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.dart';
import '../../test_util/test_utils.dart';

void main() {
  late NavigationService service;
  late MockCheckNavigationPrerequisitesUseCase mockCheckNavigationPrerequisitesUseCase;
  late MockPerformPreNavigationActionsUseCase mockPerformPreNavigationActionsUseCase;
  late MockGetWalletStateUseCase mockGetWalletStateUseCase;
  late MockNavigatorKey navigatorKey;
  late MockNavigatorState navigatorState;
  late MockBuildContext context;

  setUp(() {
    navigatorKey = MockNavigatorKey();
    navigatorState = MockNavigatorState();
    context = MockBuildContext();
    when(navigatorState.context).thenAnswer((_) => context);
    when(navigatorKey.currentState).thenReturn(navigatorState);
    // Usecases
    mockCheckNavigationPrerequisitesUseCase = MockCheckNavigationPrerequisitesUseCase();
    mockPerformPreNavigationActionsUseCase = MockPerformPreNavigationActionsUseCase();
    mockGetWalletStateUseCase = MockGetWalletStateUseCase();

    service = NavigationService(
      navigatorKey,
      mockCheckNavigationPrerequisitesUseCase,
      mockPerformPreNavigationActionsUseCase,
      mockGetWalletStateUseCase,
    );
  });

  group('handleNavigationRequest', () {
    test('When navigation prerequisites are fulfilled, trigger actual navigation', () async {
      // Allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      const navigationRequest = GenericNavigationRequest('/mock');
      await service.handleNavigationRequest(navigationRequest);

      verify(navigatorState.pushNamed(navigationRequest.destination));
    });

    test('Verify PidIssuanceNavigationRequest triggers navigation', () async {
      // Allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      final navigationRequest = NavigationRequest.pidIssuance('/mock');
      await service.handleNavigationRequest(navigationRequest);

      verify(
        navigatorState.pushNamedAndRemoveUntil(
          navigationRequest.destination,
          any,
          arguments: anyNamed('arguments'),
        ),
      );
    });

    test('Verify walletTransferTarget navRequest triggers navigation', () async {
      // Allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      final navigationRequest = NavigationRequest.walletTransferTarget();
      await service.handleNavigationRequest(navigationRequest);

      verify(
        navigatorState.pushNamedAndRemoveUntil(
          navigationRequest.destination,
          any,
          arguments: anyNamed('arguments'),
        ),
      );
    });

    test('Verify appBlocked navRequest triggers navigation', () async {
      // Allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      final navigationRequest = NavigationRequest.appBlocked(reason: .adminRequest);
      await service.handleNavigationRequest(navigationRequest);

      verify(
        navigatorState.pushNamedAndRemoveUntil(
          navigationRequest.destination,
          any,
          arguments: anyNamed('arguments'),
        ),
      );
    });

    test('Verify DisclosureNavigationRequest triggers navigation', () async {
      // Allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      final navigationRequest = NavigationRequest.disclosure(
        argument: const DisclosureScreenArgument(type: .remote('/mock', isQrCode: false)),
      );
      await service.handleNavigationRequest(navigationRequest);

      verify(
        navigatorState.pushNamedAndRemoveUntil(
          navigationRequest.destination,
          any,
          arguments: anyNamed('arguments'),
        ),
      );
    });

    test('Verify IssuanceNavigationRequest triggers navigation', () async {
      // Allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      final navigationRequest = NavigationRequest.issuance(
        argument: const IssuanceScreenArgument(isQrCode: false, uri: '/mock', issuanceType: .disclosureBasedIssuance),
      );
      await service.handleNavigationRequest(navigationRequest);

      verify(
        navigatorState.pushNamedAndRemoveUntil(
          navigationRequest.destination,
          any,
          arguments: anyNamed('arguments'),
        ),
      );
    });

    test('Verify SignNavigationRequest triggers navigation', () async {
      // Allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      final navigationRequest = NavigationRequest.sign(argument: const SignScreenArgument(uri: '/mock'));
      await service.handleNavigationRequest(navigationRequest);

      verify(
        navigatorState.pushNamedAndRemoveUntil(
          navigationRequest.destination,
          any,
          arguments: anyNamed('arguments'),
        ),
      );
    });

    test('Verify preNavigationActions are executed before the navigation occurs', () async {
      // Allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      await service.handleNavigationRequest(const GenericNavigationRequest('/mock'));

      verifyInOrder([mockPerformPreNavigationActionsUseCase.invoke(any), navigatorKey.currentState]);
    });

    test('When navigation prerequisites are not fulfilled, do not trigger actual navigation', () async {
      // Disallow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);

      await service.handleNavigationRequest(const GenericNavigationRequest('/mock'));

      // Make sure navigation was NOT triggered
      verifyNever(navigatorKey.currentState);
    });
  });

  group('processQueue', () {
    test('Verify request is not queued if not explicitly requested', () async {
      // Disallow navigation, meaning the request should be queued if that is requested
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);
      await service.handleNavigationRequest(const GenericNavigationRequest('/mock'));

      // Now allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);
      // And process any queue if it exists
      await service.processQueue();

      // Make sure navigation was NOT triggered
      verifyNever(navigatorKey.currentState);
    });

    test('Verify request is queued when requested', () async {
      // Disallow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);
      await service.handleNavigationRequest(const GenericNavigationRequest('/mock'), queueIfNotReady: true);

      // Now allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);
      // And process any queue if it exists
      await service.processQueue();

      // Make sure navigation was triggered
      verify(navigatorKey.currentState);
    });

    test(
      'Verify navigation does not occur when prerequisites are not met',
      () async {
        // Disallow navigation
        when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);

        // Queue event
        await service.handleNavigationRequest(const GenericNavigationRequest('/mock'), queueIfNotReady: true);

        // Attempt to process queue
        await service.processQueue();

        // Verify queue was not processed
        verifyZeroInteractions(navigatorKey.currentState);
      },
    );
  });

  group('hasQueuedRequest', () {
    test('Verify hasQueuedRequest is false initially', () {
      expect(service.hasQueuedRequest, isFalse);
    });

    test('Verify hasQueuedRequest is true when a request is queued', () async {
      // Disallow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);
      await service.handleNavigationRequest(const GenericNavigationRequest('/mock'), queueIfNotReady: true);

      expect(service.hasQueuedRequest, isTrue);
    });

    test('Verify hasQueuedRequest is false when a queued request is processed', () async {
      // Disallow navigation initially
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);
      await service.handleNavigationRequest(const GenericNavigationRequest('/mock'), queueIfNotReady: true);
      expect(service.hasQueuedRequest, isTrue);

      // Now allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);
      await service.processQueue();

      expect(service.hasQueuedRequest, isFalse);
    });

    test('Verify hasQueuedRequest is false when a new request is handled', () async {
      // Disallow navigation initially and queue
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);
      await service.handleNavigationRequest(const GenericNavigationRequest('/mock'), queueIfNotReady: true);
      expect(service.hasQueuedRequest, isTrue);

      // Handle a new request that is ready
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);
      await service.handleNavigationRequest(const GenericNavigationRequest('/new-mock'), queueIfNotReady: true);

      expect(service.hasQueuedRequest, isFalse);
    });
  });

  group('dialog', () {
    test('Verify no dialog is pushed when context is not mounted', () async {
      when(context.mounted).thenAnswer((_) => false);
      await service.showDialog(WalletDialogType.idleWarning);
      verifyNever(navigatorState.push(any));
    });
  });

  group('processUpdateNotification', () {
    test('Verify no dialog is shown when context is not mounted', () async {
      when(context.mounted).thenAnswer((_) => false);
      await service.processUpdateNotification(RecommendUpdateNotification());
      verifyNever(navigatorState.push(any));
    });

    // These tests use a real [MaterialApp]/[Navigator] (instead of the Mockito mocks used above), because
    // `UpdateNotificationDialog.show` needs a real BuildContext (Navigator, Overlay, Localizations) to render.
    group('with a real Navigator', () {
      late GlobalKey<NavigatorState> realNavigatorKey;
      late NavigationService realService;

      setUp(() {
        realNavigatorKey = GlobalKey<NavigatorState>();
        realService = NavigationService(
          realNavigatorKey,
          mockCheckNavigationPrerequisitesUseCase,
          mockPerformPreNavigationActionsUseCase,
          mockGetWalletStateUseCase,
        );
      });

      testWidgets('RecommendUpdateNotification shows the UpdateNotificationDialog without a countdown', (
        tester,
      ) async {
        await tester.pumpWidgetWithAppWrapper(const SizedBox(), navigatorKey: realNavigatorKey);
        unawaited(realService.processUpdateNotification(RecommendUpdateNotification()));
        await tester.pumpAndSettle();

        expect(find.byType(UpdateNotificationDialog), findsOneWidget);
        expect(
          tester.widget<UpdateNotificationDialog>(find.byType(UpdateNotificationDialog)).timeUntilBlocked,
          isNull,
        );
      });

      testWidgets('WarnUpdateNotification shows the UpdateNotificationDialog with the given countdown', (
        tester,
      ) async {
        const timeUntilBlocked = Duration(hours: 3);
        await tester.pumpWidgetWithAppWrapper(const SizedBox(), navigatorKey: realNavigatorKey);
        unawaited(realService.processUpdateNotification(WarnUpdateNotification(timeUntilBlocked: timeUntilBlocked)));
        await tester.pumpAndSettle();

        expect(find.byType(UpdateNotificationDialog), findsOneWidget);
        expect(
          tester.widget<UpdateNotificationDialog>(find.byType(UpdateNotificationDialog)).timeUntilBlocked,
          timeUntilBlocked,
        );
      });
    });
  });

  group('handleNavigationRequest blocked-dialog resolution', () {
    late GlobalKey<NavigatorState> realNavigatorKey;
    late NavigationService realService;

    setUp(() {
      realNavigatorKey = GlobalKey<NavigatorState>();
      realService = NavigationService(
        realNavigatorKey,
        mockCheckNavigationPrerequisitesUseCase,
        mockPerformPreNavigationActionsUseCase,
        mockGetWalletStateUseCase,
      );
      // Navigation is never ready in this group; we're only interested in the resulting dialog (if any).
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);
    });

    Future<void> expectBlockedDialog(
      WidgetTester tester,
      WalletState state, {
      // When null, we expect no dialog
      String Function(AppLocalizations)? genericDialogTitle,
    }) async {
      when(mockGetWalletStateUseCase.invoke()).thenAnswer((_) async => state);
      await tester.pumpWidgetWithAppWrapper(const SizedBox(), navigatorKey: realNavigatorKey);
      await realService.handleNavigationRequest(const GenericNavigationRequest('/mock'));
      await tester.pumpAndSettle();
      if (genericDialogTitle == null) {
        expect(find.byType(GenericDialog), findsNothing);
      } else {
        final l10n = await TestUtils.englishLocalizations;
        expect(tester.widget<GenericDialog>(find.byType(GenericDialog)).title, genericDialogTitle(l10n));
      }
    }

    testWidgets('WalletStateUnregistered blocks with the finishSetup dialog', (tester) async {
      await expectBlockedDialog(
        tester,
        const WalletStateUnregistered(),
        genericDialogTitle: (l10n) => l10n.finishSetupDialogTitle,
      );
    });

    testWidgets('WalletStateEmpty blocks with the finishSetup dialog', (tester) async {
      await expectBlockedDialog(
        tester,
        const WalletStateEmpty(),
        genericDialogTitle: (l10n) => l10n.finishSetupDialogTitle,
      );
    });

    testWidgets('WalletStateTransferPossible blocks with the finishTransferringDestination dialog', (tester) async {
      await expectBlockedDialog(
        tester,
        const WalletStateTransferPossible(),
        genericDialogTitle: (l10n) => l10n.finishTransferDestinationDialogTitle,
      );
    });

    testWidgets('WalletStateTransferring(source) blocks with the finishTransferringSource dialog', (tester) async {
      await expectBlockedDialog(
        tester,
        const WalletStateTransferring(.source),
        genericDialogTitle: (l10n) => l10n.finishTransferSourceDialogTitle,
      );
    });

    testWidgets('WalletStateTransferring(destination) blocks with the finishTransferringDestination dialog', (
      tester,
    ) async {
      await expectBlockedDialog(
        tester,
        const WalletStateTransferring(.destination),
        genericDialogTitle: (l10n) => l10n.finishTransferDestinationDialogTitle,
      );
    });

    testWidgets('WalletStateInDisclosureFlow blocks with the finishActiveDisclosureSession dialog', (tester) async {
      await expectBlockedDialog(
        tester,
        const WalletStateInDisclosureFlow(),
        genericDialogTitle: (l10n) => l10n.activeSessionDialogTitle,
      );
    });

    testWidgets('WalletStateInIssuanceFlow blocks with the finishActiveIssuanceSession dialog', (tester) async {
      await expectBlockedDialog(
        tester,
        const WalletStateInIssuanceFlow(),
        genericDialogTitle: (l10n) => l10n.activeIssuanceSessionDialogTitle,
      );
    });

    testWidgets('WalletStateInPinChangeFlow blocks with the finishRecoverPin dialog', (tester) async {
      await expectBlockedDialog(
        tester,
        const WalletStateInPinChangeFlow(),
        genericDialogTitle: (l10n) => l10n.finishRecoverPinDialogTitle,
      );
    });

    testWidgets('WalletStateInPinRecoveryFlow blocks with the finishRecoverPin dialog', (tester) async {
      await expectBlockedDialog(
        tester,
        const WalletStateInPinRecoveryFlow(),
        genericDialogTitle: (l10n) => l10n.finishRecoverPinDialogTitle,
      );
    });

    testWidgets('WalletStateReady does not block with a dialog', (tester) async {
      await expectBlockedDialog(tester, const WalletStateReady());
    });

    testWidgets('WalletStateLocked does not block with a dialog', (tester) async {
      await expectBlockedDialog(tester, const WalletStateLocked(WalletStateReady()));
    });

    testWidgets('WalletStateBlocked does not block with a dialog', (tester) async {
      await expectBlockedDialog(
        tester,
        const WalletStateBlocked(.requiresAppUpdate, canRegisterNewAccount: false),
      );
    });
  });

  group('onCoreError', () {
    test('When CoreAccountRevokedError is received, navigate to app blocked screen', () async {
      // Allow navigation (empty prerequisites for appBlocked)
      when(mockCheckNavigationPrerequisitesUseCase.invoke([])).thenAnswer((_) async => true);

      final error = CoreAccountRevokedError(
        'Account revoked',
        revocationData: RevocationData(
          revocationReason: RevocationReason.adminRequest,
          canRegisterNewAccount: false,
        ),
      );

      await service.onCoreError(error);

      final expectedRequest = NavigationRequest.appBlocked(reason: RevocationReason.adminRequest);

      verify(
        navigatorState.pushNamedAndRemoveUntil(
          expectedRequest.destination,
          any,
          arguments: expectedRequest.argument,
        ),
      ).called(1);
    });

    test('When CoreAccountRevokedError with different reason is received, navigate with that reason', () async {
      // Allow navigation (empty prerequisites for appBlocked)
      when(mockCheckNavigationPrerequisitesUseCase.invoke([])).thenAnswer((_) async => true);

      final error = CoreAccountRevokedError(
        'Account revoked',
        revocationData: RevocationData(
          revocationReason: RevocationReason.solutionCompromised,
          canRegisterNewAccount: true,
        ),
      );

      await service.onCoreError(error);

      final expectedRequest = NavigationRequest.appBlocked(reason: RevocationReason.solutionCompromised);

      verify(
        navigatorState.pushNamedAndRemoveUntil(
          expectedRequest.destination,
          any,
          arguments: expectedRequest.argument,
        ),
      ).called(1);
    });

    test('When CoreStateError is received, navigate to the invariant error screen', () async {
      // Allow navigation (empty prerequisites for invariantError)
      when(mockCheckNavigationPrerequisitesUseCase.invoke([])).thenAnswer((_) async => true);

      const error = CoreStateError('boom');

      await service.onCoreError(error);

      final expectedRequest = NavigationRequest.invariantError(code: 'boom');

      verify(
        navigatorState.pushNamedAndRemoveUntil(
          expectedRequest.destination,
          any,
          arguments: expectedRequest.argument,
        ),
      ).called(1);
    });

    test('When other CoreError is received, do nothing', () async {
      const error = CoreGenericError('Some error');

      await service.onCoreError(error);

      verifyZeroInteractions(navigatorState);
    });
  });
}
