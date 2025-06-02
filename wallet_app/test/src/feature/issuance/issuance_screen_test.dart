import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/issuance/issuance_repository.dart';
import 'package:wallet/src/data/repository/wallet/wallet_repository.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/flow_progress.dart';
import 'package:wallet/src/domain/model/policy/organization_policy.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/app/check_is_app_initialized_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/is_biometric_login_enabled_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/disclose_for_issuance_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import 'package:wallet/src/feature/common/screen/request_details_screen.dart';
import 'package:wallet/src/feature/issuance/bloc/issuance_bloc.dart';
import 'package:wallet/src/feature/issuance/issuance_screen.dart';
import 'package:wallet/src/feature/issuance/page/issuance_review_cards_page.dart';
import 'package:wallet/src/feature/organization/approve/organization_approve_page.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/util/manager/biometric_unlock_manager.dart';
import 'package:wallet/src/util/mapper/context_mapper.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../mocks/wallet_mocks.mocks.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

class MockIssuanceBloc extends MockBloc<IssuanceEvent, IssuanceState> implements IssuanceBloc {}

void main() {
  group('goldens', () {
    testGoldens('IssuanceInitial Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceInitial(),
        ),
      );
      await screenMatchesGolden('initial.light');
    });

    testGoldens('IssuanceInitial Dark Landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceInitial(),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('initial.dark.landscape');
    });

    testGoldens('IssuanceLoadInProgress Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceLoadInProgress(FlowProgress(currentStep: 3, totalSteps: kIssuanceSteps)),
        ),
      );
      await screenMatchesGolden('loading.light');
    });

    testGoldens('IssuanceLoadInProgress Dark Landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceLoadInProgress(FlowProgress(currentStep: 4, totalSteps: kIssuanceSteps)),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('loading.dark.landscape');
    });

    testGoldens('IssuanceCheckOrganization Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          IssuanceCheckOrganization(
            organization: WalletMockData.organization,
            policy: WalletMockData.policy,
            requestedAttributes: {
              WalletMockData.card: [WalletMockData.textDataAttribute],
            },
          ),
        ),
      );
      await screenMatchesGolden('check_organization.light');
    });

    testGoldens('IssuanceCheckOrganization Dark Landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          IssuanceCheckOrganization(
            organization: WalletMockData.organization,
            policy: WalletMockData.policy,
            requestedAttributes: {
              WalletMockData.card: [WalletMockData.textDataAttribute],
            },
          ),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('check_organization.dark.landscape');
    });

    testGoldens('IssuanceMissingAttributes Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          IssuanceMissingAttributes(
            organization: WalletMockData.organization,
            missingAttributes: [MissingAttribute(label: 'BSN'.untranslated)],
          ),
        ),
      );
      await screenMatchesGolden('missing_attributes.light');
    });

    testGoldens('IssuanceMissingAttributes Dark Landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          IssuanceMissingAttributes(
            organization: WalletMockData.organization,
            missingAttributes: [MissingAttribute(label: 'BSN'.untranslated)],
          ),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('missing_attributes.dark.landscape');
    });
    testGoldens('IssuanceProvidePinForDisclosure Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceProvidePinForDisclosure(),
        ),
        providers: [
          RepositoryProvider<DiscloseForIssuanceUseCase>(create: (c) => MockDiscloseForIssuanceUseCase()),
        ],
      );
      await screenMatchesGolden('provide_pin.disclosure.light');
    });

    testGoldens('IssuanceProvidePinForDisclosure Dark Landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceProvidePinForDisclosure(),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [
          RepositoryProvider<DiscloseForIssuanceUseCase>(create: (c) => MockDiscloseForIssuanceUseCase()),
        ],
      );
      await screenMatchesGolden('provide_pin.disclosure.dark.landscape');
    });

    testGoldens('IssuanceReviewCards Light - Single card', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          IssuanceReviewCards.init(
            cards: [WalletMockData.card],
          ),
        ),
      );
      await screenMatchesGolden('review_card.light');
    });

    testGoldens('IssuanceReviewCards Dark Landscape - Multi card', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          IssuanceReviewCards.init(
            cards: [WalletMockData.card, WalletMockData.altCard],
          ),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('review_multi_cards.dark.landscape');
    });

    testGoldens('IssuanceProvidePinForIssuance Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          IssuanceProvidePinForIssuance(
            cards: [WalletMockData.card],
          ),
        ),
        providers: [
          RepositoryProvider<DiscloseForIssuanceUseCase>(create: (c) => MockDiscloseForIssuanceUseCase()),
          RepositoryProvider<IssuanceRepository>(create: (c) => MockIssuanceRepository()),
        ],
      );
      await screenMatchesGolden('provide_pin.issuance.light');
    });

    testGoldens('IssuanceProvidePinForIssuance Dark Landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceProvidePinForIssuance(
            cards: [],
          ),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [
          RepositoryProvider<DiscloseForIssuanceUseCase>(create: (c) => MockDiscloseForIssuanceUseCase()),
          RepositoryProvider<IssuanceRepository>(create: (c) => MockIssuanceRepository()),
        ],
      );
      await screenMatchesGolden('provide_pin.issuance.dark.landscape');
    });

    testGoldens('IssuanceCompleted Light - Single card', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          IssuanceCompleted(addedCards: [WalletMockData.card]),
        ),
        providers: [
          RepositoryProvider<IssuanceRepository>(create: (c) => MockIssuanceRepository()),
        ],
      );
      await screenMatchesGolden('completed.light');
    });

    testGoldens('IssuanceCompleted Dark Landscape - Multi card, No returnUrl', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          IssuanceCompleted(
            addedCards: [WalletMockData.card, WalletMockData.altCard],
          ),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('completed.multi_card.dark.landscape');
    });

    testGoldens('IssuanceStopped Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceStopped(),
        ),
      );
      await screenMatchesGolden('stopped.light');
    });

    testGoldens('IssuanceStopped Light - with return url', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceStopped(returnUrl: 'https://example.org'),
        ),
      );
      await screenMatchesGolden('stopped.return_url.light');
    });

    testGoldens('IssuanceStopped Dark Landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceStopped(),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('stopped.dark.landscape');
    });

    testGoldens('IssuanceGenericError Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceGenericError(
            error: GenericError('test', sourceError: CoreGenericError('test')),
          ),
        ),
      );
      await screenMatchesGolden('error.light');
    });

    testGoldens('IssuanceGenericError Dark Landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceGenericError(
            error: GenericError('test', sourceError: CoreGenericError('test')),
          ),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('error.dark.landscape');
    });

    testGoldens('IssuanceNoCardsRetrieved Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          IssuanceNoCardsRetrieved(
            organization: WalletMockData.organization,
          ),
        ),
      );
      await screenMatchesGolden('no_cards_retrieved.light');
    });

    testGoldens('IssuanceNoCardsRetrieved Dark Landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          IssuanceNoCardsRetrieved(
            organization: WalletMockData.organization,
          ),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('no_cards_retrieved.dark.landscape');
    });

    testGoldens('IssuanceExternalScannerError Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceExternalScannerError(error: GenericError('test', sourceError: 'test')),
        ),
      );
      await screenMatchesGolden('external_scanner_error.light');
    });

    testGoldens('IssuanceExternalScannerError Dark Landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceExternalScannerError(error: GenericError('test', sourceError: 'test')),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('external_scanner_error.dark.landscape');
    });

    testGoldens('IssuanceNetworkError Light - no internet', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceNetworkError(error: GenericError('test', sourceError: 'test'), hasInternet: false),
        ),
      );
      await screenMatchesGolden('network_error.light');
    });

    testGoldens('IssuanceNetworkError Dark Landscape - internet available', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceNetworkError(error: GenericError('test', sourceError: 'test'), hasInternet: true),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('network_error.dark.landscape');
    });

    testGoldens('IssuanceSessionExpired Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceSessionExpired(
            error: GenericError('test', sourceError: 'test'),
            isCrossDevice: false,
            canRetry: true,
            returnUrl: 'https://example.org',
          ),
        ),
      );
      await screenMatchesGolden('session_expired.light');
    });

    testGoldens('IssuanceSessionExpired Dark Landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceSessionExpired(
            error: GenericError('test', sourceError: 'test'),
            isCrossDevice: true,
            canRetry: false,
            returnUrl: 'https://example.org',
          ),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('session_expired.dark.landscape');
    });

    testGoldens('IssuanceCancelledSessionError Light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceSessionCancelled(error: GenericError('test', sourceError: 'test')),
        ),
      );
      await screenMatchesGolden('cancelled_session_error.light');
    });

    testGoldens('IssuanceCancelledSessionError Dark Landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          const IssuanceSessionCancelled(error: GenericError('test', sourceError: 'test')),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('cancelled_session_error.dark.landscape');
    });

    testGoldens('StopSheet - light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          IssuanceReviewCards.init(cards: [WalletMockData.card]),
        ),
      );

      // Press the stop button
      await tester.tap(find.byKey(kReviewCardsDeclineButtonKey));
      await tester.pumpAndSettle();

      await screenMatchesGolden('stop_sheet.light');
    });
  });

  group('widgets', () {
    testWidgets('continue cta is visible when issuance is completed', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          IssuanceCompleted(
            addedCards: [WalletMockData.card, WalletMockData.altCard],
          ),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.walletPersonalizeSuccessPageContinueCta), findsOneWidget);
    });

    testWidgets('Pressing show details on check organization page opens request details screen', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          IssuanceCheckOrganization(
            organization: WalletMockData.organization,
            policy: WalletMockData.policy,
            requestedAttributes: {
              WalletMockData.card: [
                WalletMockData.textDataAttribute,
                WalletMockData.textDataAttribute,
              ],
            },
          ),
        ),
        providers: [
          RepositoryProvider<WalletRepository>(
            create: (_) {
              final mock = MockWalletRepository();
              when(mock.isLockedStream).thenAnswer((_) => Stream.value(false));
              return mock;
            },
          ),
          RepositoryProvider<IsWalletInitializedUseCase>(create: (_) => MockIsWalletInitializedUseCase()),
          RepositoryProvider<IsBiometricLoginEnabledUseCase>(create: (_) => MockIsBiometricLoginEnabledUseCase()),
          RepositoryProvider<BiometricUnlockManager>(create: (_) => MockBiometricUnlockManager()),
          RepositoryProvider<UnlockWalletWithPinUseCase>(create: (_) => MockUnlockWalletWithPinUseCase()),
          RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(create: (_) => MockContextMapper()),
        ],
      );

      // Press the show details button
      await tester.tap(find.byKey(kShowDetailsButtonKey));
      await tester.pumpAndSettle();

      expect(find.byType(RequestDetailsScreen), findsOneWidget);
    });

    testWidgets('When user rejects cards on review page, the stop sheet is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const IssuanceScreen().withState<IssuanceBloc, IssuanceState>(
          MockIssuanceBloc(),
          IssuanceReviewCards.init(cards: [WalletMockData.card]),
        ),
      );

      // Press the stop button
      await tester.tap(find.byKey(kReviewCardsDeclineButtonKey));
      await tester.pumpAndSettle();

      // Verify the expected stop sheet description text is shown
      final l10n = await TestUtils.englishLocalizations;
      final organizationName = l10n.organizationFallbackName;
      expect(find.text(l10n.issuanceStopSheetDescription(organizationName), findRichText: true), findsOneWidget);
    });
  });
}
