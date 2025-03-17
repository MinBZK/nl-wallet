import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/wallet/wallet_repository.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_session_type.dart';
import 'package:wallet/src/domain/model/policy/organization_policy.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/app/check_is_app_initialized_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/is_biometric_login_enabled_usecase.dart';
import 'package:wallet/src/domain/usecase/disclosure/accept_disclosure_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import 'package:wallet/src/feature/common/widget/button/icon/close_icon_button.dart';
import 'package:wallet/src/feature/common/widget/centered_loading_indicator.dart';
import 'package:wallet/src/feature/disclosure/bloc/disclosure_bloc.dart';
import 'package:wallet/src/feature/disclosure/disclosure_screen.dart';
import 'package:wallet/src/feature/disclosure/page/disclosure_confirm_data_attributes_page.dart';
import 'package:wallet/src/feature/disclosure/page/disclosure_confirm_pin_page.dart';
import 'package:wallet/src/feature/disclosure/page/disclosure_missing_attributes_page.dart';
import 'package:wallet/src/feature/disclosure/page/disclosure_stopped_page.dart';
import 'package:wallet/src/feature/disclosure/widget/disclosure_stop_sheet.dart';
import 'package:wallet/src/feature/login/login_detail_screen.dart';
import 'package:wallet/src/feature/organization/approve/organization_approve_page.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/pin/widget/pin_keyboard.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/util/manager/biometric_unlock_manager.dart';
import 'package:wallet/src/util/mapper/context_mapper.dart';
import 'package:wallet/src/util/mapper/policy/policy_body_text_mapper.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../mocks/wallet_mocks.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';
import '../pin/pin_page_test.dart';

class MockDisclosureBloc extends MockBloc<DisclosureEvent, DisclosureState> implements DisclosureBloc {
  @override
  final bool isLoginFlow;

  MockDisclosureBloc({this.isLoginFlow = false});
}

void main() {
  group('goldens', () {
    testGoldens('DisclosureInitial Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              const DisclosureInitial(),
            ),
            name: 'initial',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'initial.light');
    });

    testGoldens('DisclosureLoadInProgress Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              DisclosureLoadInProgress(),
            ),
            name: 'load_in_progress',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'load_in_progress.light');
    });

    testGoldens('DisclosureCheckOrganization Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              DisclosureCheckOrganization(
                relyingParty: WalletMockData.organization,
                originUrl: 'http://origin.org',
                sharedDataWithOrganizationBefore: true,
                sessionType: DisclosureSessionType.crossDevice,
              ),
            ),
            name: 'check_organization',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'check_organization.light');
    });

    testGoldens('DisclosureGenericError Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              const DisclosureGenericError(error: GenericError('generic', sourceError: 'test')),
            ),
            name: 'generic_error',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'generic_error.light');
    });

    testGoldens('DisclosureConfirmPin Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: RepositoryProvider<AcceptDisclosureUseCase>.value(
              value: MockAcceptDisclosureUseCase(),
              child: const DisclosureScreen()
                  .withState<DisclosureBloc, DisclosureState>(
                    MockDisclosureBloc(),
                    DisclosureConfirmPin(relyingParty: WalletMockData.organization),
                  )
                  .withState<PinBloc, PinState>(
                    MockPinBloc(),
                    const PinEntryInProgress(0),
                  ),
            ),
            name: 'confirm_pin',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'confirm_pin.light');
    });

    testGoldens('DisclosureMissingAttributes Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              DisclosureMissingAttributes(
                relyingParty: WalletMockData.organization,
                missingAttributes: [MissingAttribute(label: 'missing'.untranslated)],
              ),
            ),
            name: 'missing_attributes',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'missing_attributes.light');
    });

    testGoldens('DisclosureConfirmDataAttributes Dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              DisclosureConfirmDataAttributes(
                relyingParty: WalletMockData.organization,
                requestedAttributes: {
                  WalletMockData.card: [WalletMockData.textDataAttribute],
                },
                requestPurpose: 'Sample reason'.untranslated,
                policy: WalletMockData.policy,
              ),
            ),
            name: 'confirm_data_attributes',
          ),
        wrapper: walletAppWrapper(
          brightness: Brightness.dark,
          providers: [
            RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(create: (c) => PolicyBodyTextMapper()),
          ],
        ),
      );
      await screenMatchesGolden(tester, 'confirm_data_attributes.dark');
    });

    testGoldens('DisclosureConfirmDataAttributes - full page', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        surfaceSize: const Size(375, 1100 /* tall to fit all content */),
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          DisclosureConfirmDataAttributes(
            relyingParty: WalletMockData.organization,
            requestedAttributes: {
              WalletMockData.card: [WalletMockData.textDataAttribute],
            },
            requestPurpose: 'Sample reason'.untranslated,
            policy: WalletMockData.policy,
          ),
        ),
        brightness: Brightness.dark,
        providers: [
          RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(create: (c) => PolicyBodyTextMapper()),
        ],
      );
      await screenMatchesGolden(tester, 'confirm_data_attributes');
    });

    testGoldens('DisclosureSuccess Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              DisclosureSuccess(relyingParty: WalletMockData.organization, event: WalletMockData.disclosureEvent),
            ),
            name: 'success',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'success.light');
    });

    testGoldens('DisclosureStopped Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              DisclosureStopped(organization: WalletMockData.organization),
            ),
            name: 'stopped',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'stopped.light');
    });

    testGoldens('DisclosureLeftFeedback Light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
              MockDisclosureBloc(),
              const DisclosureLeftFeedback(),
            ),
            name: 'left_feedback',
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'left_feedback.light');
    });

    testGoldens('Disclosure Stop Sheet Light', (tester) async {
      // Inflate a state with showStopConfirmation = true.
      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          DisclosureCheckOrganization(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            sharedDataWithOrganizationBefore: true,
            sessionType: DisclosureSessionType.crossDevice,
          ),
        ),
      );
      // Find and press the close button
      final closeButtonFinder = find.byKey(kCloseIconButtonKey);
      await tester.tap(closeButtonFinder);
      await tester.pumpAndSettle();

      await screenMatchesGolden(tester, 'stop_sheet.light');
    });
  });

  group('widgets', () {
    testWidgets('when cross-device session; fraud warning is shown on organization approve page', (tester) async {
      const originUrl = 'http://origin.org';

      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          DisclosureCheckOrganization(
            relyingParty: WalletMockData.organization,
            originUrl: originUrl,
            sharedDataWithOrganizationBefore: true,
            sessionType: DisclosureSessionType.crossDevice,
          ),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.organizationApprovePageFraudInfoPart1, findRichText: true), findsOneWidget);

      final leadingPart = l10n.organizationApprovePageFraudInfoPart2(originUrl).split(originUrl).first.trim();
      final trailingPart = l10n.organizationApprovePageFraudInfoPart2(originUrl).split(originUrl).last.trim();
      // We match on the longest piece of text (before/after the embedded [originUrl], to avoid only matching on a "." which could cause multiple hits (e.g. with current translations).
      // We consider onlyu checking for the longest part sufficient because the main thing we want to verify is that this warning is visible (in cross device flows).
      final matchOn = leadingPart.length >= trailingPart.length ? leadingPart : trailingPart;
      expect(find.textContaining(matchOn, findRichText: true), findsOneWidget);

      // Verify the originUrl is visible, now in a dedicated widget (which is why this test is more convoluted in the first place,
      // as the new [UrlSpan] is not directly matchable using the findRichText flag.
      expect(find.text(originUrl), findsOneWidget);
    });

    testWidgets('when same-device session; fraud warning is NOT shown on organization approve page', (tester) async {
      const originUrl = 'http://origin.org';

      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          DisclosureCheckOrganization(
            relyingParty: WalletMockData.organization,
            originUrl: 'http://origin.org',
            sharedDataWithOrganizationBefore: true,
            sessionType: DisclosureSessionType.sameDevice,
          ),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.organizationApprovePageFraudInfoPart1, findRichText: true), findsNothing);
      expect(find.text(l10n.organizationApprovePageFraudInfoPart2(originUrl), findRichText: true), findsNothing);
    });

    testWidgets('history button is shown on the success page', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          DisclosureSuccess(relyingParty: WalletMockData.organization, event: WalletMockData.disclosureEvent),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.disclosureSuccessPageShowHistoryCta), findsOneWidget);
    });

    testWidgets('DisclosureScreen shows the no internet error for DisclosureNetworkError(hasInternet=false)',
        (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          const DisclosureNetworkError(
            error: NetworkError(hasInternet: false, sourceError: 'no internet'),
            hasInternet: false,
          ),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;

      await tester.pumpAndSettle();

      // Verify the 'no internet' title is shown
      final noInternetHeadlineFinder = find.text(l10n.errorScreenNoInternetHeadline);
      expect(noInternetHeadlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'close' cta is shown
      final closeCtaFinder = find.text(l10n.generalClose);
      expect(closeCtaFinder, findsOneWidget);

      // Verify the 'close' icon is shown
      final closeIconFinder = find.byIcon(Icons.close_outlined);
      expect(closeIconFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('DisclosureScreen shows the server error for DisclosureNetworkError(hasInternet=true)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          const DisclosureNetworkError(
            error: NetworkError(hasInternet: true, sourceError: 'server'),
            hasInternet: true,
          ),
        ),
      );
      final l10n = await TestUtils.englishLocalizations;

      await tester.pumpAndSettle();

      // Verify the 'no internet' title is shown
      final noInternetHeadlineFinder = find.text(l10n.errorScreenServerHeadline);
      expect(noInternetHeadlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'close' cta is shown
      final closeCtaFinder = find.text(l10n.generalClose);
      expect(closeCtaFinder, findsOneWidget);

      // Verify the 'close' icon is shown
      final closeIconFinder = find.byIcon(Icons.close_outlined);
      expect(closeIconFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('DisclosureScreen shows the generic error for CoreGenericError', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          const DisclosureGenericError(error: GenericError('generic', sourceError: 'test')),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'something went wrong' title is shown
      final headlineFinder = find.text(l10n.errorScreenGenericHeadline);
      expect(headlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'close' cta is shown
      final closeCtaFinder = find.text(l10n.generalClose);
      expect(closeCtaFinder, findsOneWidget);

      // Verify the 'close' icon is shown
      final closeIconFinder = find.byIcon(Icons.close_outlined);
      expect(closeIconFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('DisclosureScreen shows session expired for DisclosureSessionExpired', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          const DisclosureSessionExpired(
            error: SessionError(
              state: SessionState.expired,
              crossDevice: SessionType.crossDevice,
              canRetry: false,
              sourceError: 'test',
            ),
            canRetry: false,
            isCrossDevice: false,
          ),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'session expired' title is shown
      final headlineFinder = find.text(l10n.errorScreenSessionExpiredHeadline);
      expect(headlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'close' cta is shown
      final closeCtaFinder = find.text(l10n.generalClose);
      expect(closeCtaFinder, findsOneWidget);

      // Verify the 'close' icon is shown
      final closeIconFinder = find.byIcon(Icons.close_outlined);
      expect(closeIconFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('DisclosureScreen shows loading text for DisclosureInitial', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          const DisclosureInitial(),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.disclosureLoadingTitle), findsOneWidget);
      expect(find.text(l10n.disclosureLoadingSubtitle), findsOneWidget);
    });

    testWidgets('DisclosureScreen shows loader for DisclosureLoadInProgress', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
          MockDisclosureBloc(),
          DisclosureLoadInProgress(),
        ),
      );

      expect(find.byType(CenteredLoadingIndicator), findsOneWidget);
    });

    testWidgets(
      'DisclosureScreen navigates to OrganizationDetailScreen when show details is pressed',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
            MockDisclosureBloc(),
            DisclosureCheckOrganization(
              relyingParty: WalletMockData.organization,
              sharedDataWithOrganizationBefore: false,
              originUrl: '',
              sessionType: DisclosureSessionType.sameDevice,
            ),
          ),
          providers: [
            RepositoryProvider<WalletRepository>(
              create: (_) {
                // Make sure the 'locked' overlay is not shown after navigating to OrgDetailsScreen.
                final mockRepo = MockWalletRepository();
                when(mockRepo.isLockedStream).thenAnswer((_) => Stream.value(false));
                return mockRepo;
              },
            ),
            RepositoryProvider<PinBloc>(create: (_) => MockPinBloc()),
            RepositoryProvider<UnlockWalletWithPinUseCase>(create: (_) => MockUnlockWalletWithPinUseCase()),
            RepositoryProvider<IsWalletInitializedUseCase>(create: (_) => MockIsWalletInitializedUseCase()),
            RepositoryProvider<IsBiometricLoginEnabledUseCase>(create: (_) => MockIsBiometricLoginEnabledUseCase()),
            RepositoryProvider<BiometricUnlockManager>(create: (c) => MockBiometricUnlockManager()),
          ],
        );

        final l10n = await TestUtils.englishLocalizations;
        final title = l10n.organizationApprovePageGenericTitle(WalletMockData.organization.displayName.testValue);
        expect(find.textContaining(title), findsAtLeast(1));

        // Navigate away
        await tester.tap(find.text(l10n.organizationApprovePageMoreInfoCta));
        await tester.pumpAndSettle();

        // DisclosureScreen title should no longer be visible
        expect(find.text(title), findsNothing);
        // Organization detail screen title should be visible
        final organizationDetailScreenTitle =
            l10n.organizationDetailScreenTitle(WalletMockData.organization.displayName.testValue);
        expect(find.text(organizationDetailScreenTitle), findsAtLeast(1));
      },
    );

    testWidgets(
      'DisclosureScreen with OrganizationForLogin state shows OrganizationApprovePage with Login copy',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
            MockDisclosureBloc(isLoginFlow: true),
            DisclosureCheckOrganizationForLogin(
              relyingParty: WalletMockData.organization,
              originUrl: 'originUrl',
              sessionType: DisclosureSessionType.crossDevice,
              policy: WalletMockData.policy,
              requestedAttributes: const {},
              sharedDataWithOrganizationBefore: false,
            ),
          ),
        );

        final l10n = await TestUtils.englishLocalizations;
        expect(find.byType(OrganizationApprovePage), findsOneWidget);
        final loginTitle = l10n.organizationApprovePageLoginTitle(WalletMockData.organization.displayName.testValue);
        expect(find.text(loginTitle), findsOneWidget);
      },
    );

    testWidgets(
      'DisclosureScreen with OrganizationForLogin navigates to LoginDetailScreen when show details is pressed',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
            MockDisclosureBloc(isLoginFlow: true),
            DisclosureCheckOrganizationForLogin(
              relyingParty: WalletMockData.organization,
              originUrl: 'originUrl',
              sessionType: DisclosureSessionType.crossDevice,
              policy: WalletMockData.policy,
              requestedAttributes: const {},
              sharedDataWithOrganizationBefore: false,
            ),
          ),
          providers: [
            RepositoryProvider<WalletRepository>(
              create: (_) {
                // Make sure the 'locked' overlay is not shown after navigating to LoginDetailScreen.
                final mockRepo = MockWalletRepository();
                when(mockRepo.isLockedStream).thenAnswer((_) => Stream.value(false));
                return mockRepo;
              },
            ),
            RepositoryProvider<PinBloc>(create: (_) => MockPinBloc()),
            RepositoryProvider<IsWalletInitializedUseCase>(create: (_) => MockIsWalletInitializedUseCase()),
            RepositoryProvider<UnlockWalletWithPinUseCase>(create: (_) => MockUnlockWalletWithPinUseCase()),
            RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(create: (c) => PolicyBodyTextMapper()),
            RepositoryProvider<IsBiometricLoginEnabledUseCase>(create: (c) => MockIsBiometricLoginEnabledUseCase()),
            RepositoryProvider<BiometricUnlockManager>(create: (c) => MockBiometricUnlockManager()),
          ],
        );

        final l10n = await TestUtils.englishLocalizations;
        await tester.tap(find.text(l10n.organizationApprovePageMoreInfoLoginCta));
        await tester.pumpAndSettle();
        expect(find.byType(LoginDetailScreen), findsOneWidget);
      },
    );

    testWidgets(
      'DisclosureScreen with DisclosureMissingAttributes displays the missing attributes',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
            MockDisclosureBloc(),
            DisclosureMissingAttributes(
              relyingParty: WalletMockData.organization,
              missingAttributes: [
                WalletMockData.textDataAttribute,
                WalletMockData.textDataAttribute,
              ],
            ),
          ),
        );

        expect(find.byType(DisclosureMissingAttributesPage), findsOneWidget);
        expect(find.text(WalletMockData.textDataAttribute.label.testValue), findsNWidgets(2));
      },
    );

    testWidgets(
      'DisclosureScreen with DisclosureConfirmDataAttributes displays the attributes',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
            MockDisclosureBloc(),
            DisclosureConfirmDataAttributes(
              relyingParty: WalletMockData.organization,
              requestPurpose: 'test purpose'.untranslated,
              requestedAttributes: {WalletMockData.card: WalletMockData.card.attributes},
              policy: WalletMockData.policy,
            ),
          ),
          providers: [
            RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(create: (c) => PolicyBodyTextMapper()),
          ],
        );

        expect(find.byType(DisclosureConfirmDataAttributesPage), findsOneWidget);
        for (final attribute in WalletMockData.card.attributes) {
          expect(find.text(attribute.label.testValue), findsOneWidget);
        }
        expect(find.text('test purpose'), findsOneWidget);
      },
    );

    testWidgets(
      'DisclosureScreen with DisclosureConfirmPin shows the PinKeyboard',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
            MockDisclosureBloc(),
            DisclosureConfirmPin(relyingParty: WalletMockData.organization),
          ),
          providers: [
            RepositoryProvider<AcceptDisclosureUseCase>(create: (c) => MockAcceptDisclosureUseCase()),
          ],
        );

        expect(find.byType(DisclosureConfirmPinPage), findsOneWidget);
        expect(find.byType(PinKeyboard), findsOneWidget);
      },
    );

    testWidgets(
      'DisclosureScreen with DisclosureStopped shows the stopped page',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
            MockDisclosureBloc(),
            DisclosureStopped(organization: WalletMockData.organization),
          ),
        );

        expect(find.byType(DisclosureStoppedPage), findsOneWidget);
      },
    );

    testWidgets(
      'DisclosureScreen shows DisclosureStopSheet when stop is pressed',
      (tester) async {
        final mockDisclosureBloc = MockDisclosureBloc();
        await tester.pumpWidgetWithAppWrapper(
          const DisclosureScreen().withState<DisclosureBloc, DisclosureState>(
            mockDisclosureBloc,
            DisclosureLoadInProgress(),
          ),
        );

        await tester.tap(find.byType(CloseIconButton));
        await tester.pumpAndSettle();

        expect(find.byType(DisclosureStopSheet), findsOneWidget);
      },
    );
  });
}
