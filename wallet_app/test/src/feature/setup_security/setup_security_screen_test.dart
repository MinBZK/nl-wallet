import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';
import 'package:wallet/src/feature/setup_security/bloc/setup_security_bloc.dart';
import 'package:wallet/src/feature/setup_security/setup_security_screen.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/device_utils.dart';
import '../../util/test_utils.dart';

class MockSetupSecurityBloc extends MockBloc<SetupSecurityEvent, SetupSecurityState> implements SetupSecurityBloc {}

void main() {
  group('goldens', () {
    testGoldens('SetupSecuritySelectPinInProgress light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilder
          ..addScenario(
            name: '3 digits',
            widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
              MockSetupSecurityBloc(),
              const SetupSecuritySelectPinInProgress(3),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'in_progress.light');
    });

    testGoldens('SetupSecuritySelectPinFailed (sequentialDigits) light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilder
          ..addScenario(
            name: 'error state',
            widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
              MockSetupSecurityBloc(),
              const SetupSecuritySelectPinFailed(reason: PinValidationError.sequentialDigits),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'error.light');
    });

    testGoldens('SetupSecurityPinConfirmationFailed retry NOT allowed light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilder
          ..addScenario(
            name: 'setup failed',
            widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
              MockSetupSecurityBloc(),
              const SetupSecurityPinConfirmationFailed(retryAllowed: false),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'pin_confirmation_failed.light');
    });

    testGoldens('SetupSecurityPinConfirmationInProgress light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilder
          ..addScenario(
            widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
              MockSetupSecurityBloc(),
              const SetupSecurityPinConfirmationInProgress(6),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'pin_confirmation_in_progress.light');
    });

    testGoldens('SetupSecurityCompleted light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilder
          ..addScenario(
            widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
              MockSetupSecurityBloc(),
              SetupSecurityCompleted(),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'completed.light');
    });

    testGoldens('SetupSecurityPinConfirmationFailed retry allowed dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilder
          ..addScenario(
            widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
              MockSetupSecurityBloc(),
              const SetupSecurityPinConfirmationFailed(retryAllowed: true),
            ),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'pin_confirmation_failed.dark');
    });

    testGoldens('SetupSecurityCreatingWallet light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilder
          ..addScenario(
            widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
              MockSetupSecurityBloc(),
              SetupSecurityCreatingWallet(),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'creating_wallet.light');
    });

    testGoldens('SetupSecuritySelectPinFailed (tooFewUniqueDigits) dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilder
          ..addScenario(
            name: 'error state',
            widget: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
              MockSetupSecurityBloc(),
              const SetupSecuritySelectPinFailed(reason: PinValidationError.tooFewUniqueDigits),
            ),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'error.dark');
    });
  });

  group('widgets', () {
    testWidgets('SetupSecurityScreen shows the correct title for SetupSecuritySelectPinInProgress state',
        (tester) async {
      await tester.pumpWidget(
        WalletAppTestWidget(
          child: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
            MockSetupSecurityBloc(),
            const SetupSecuritySelectPinInProgress(0),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the title is shown
      final titleFinder = find.text(l10n.setupSecuritySelectPinPageTitle);
      expect(titleFinder, findsOneWidget);
    });

    testWidgets('SetupSecurityScreen shows the correct title for SetupSecurityPinConfirmationInProgress state',
        (tester) async {
      await tester.pumpWidget(
        WalletAppTestWidget(
          child: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
            MockSetupSecurityBloc(),
            const SetupSecurityPinConfirmationInProgress(0),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the title is shown
      final titleFinder = find.text(l10n.setupSecurityConfirmationPageTitle);
      expect(titleFinder, findsOneWidget);
    });

    testWidgets('SetupSecurityScreen shows the no internet error for SetupSecurityNetworkError(hasInternet=false)',
        (tester) async {
      await tester.pumpWidget(
        WalletAppTestWidget(
          child: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
            MockSetupSecurityBloc(),
            const SetupSecurityNetworkError(hasInternet: false, error: CoreNetworkError('no internet')),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'no internet' title is shown
      final noInternetHeadlineFinder = find.text(l10n.errorScreenNoInternetHeadline);
      expect(noInternetHeadlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'try again' cta is shown
      final tryAgainCtaFinder = find.text(l10n.generalRetry);
      expect(tryAgainCtaFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('SetupSecurityScreen shows the server error for SetupSecurityNetworkError(hasInternet=true)',
        (tester) async {
      await tester.pumpWidget(
        WalletAppTestWidget(
          child: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
            MockSetupSecurityBloc(),
            const SetupSecurityNetworkError(hasInternet: true, error: CoreNetworkError('server')),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'server error' title is shown
      final noInternetHeadlineFinder = find.text(l10n.errorScreenServerHeadline);
      expect(noInternetHeadlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'try again' cta is shown
      final tryAgainCtaFinder = find.text(l10n.generalRetry);
      expect(tryAgainCtaFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('SetupSecurityScreen shows the generic error for SetupSecurityGenericError state', (tester) async {
      await tester.pumpWidget(
        WalletAppTestWidget(
          child: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
            MockSetupSecurityBloc(),
            const SetupSecurityGenericError(error: CoreGenericError('generic')),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'something went wrong' title is shown
      final headlineFinder = find.text(l10n.errorScreenGenericHeadline);
      expect(headlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'try again' cta is shown
      final tryAgainCtaFinder = find.text(l10n.generalRetry);
      expect(tryAgainCtaFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('SetupSecurityScreen shows the device incompatible error for SetupSecurityDeviceIncompatibleError',
        (tester) async {
      await tester.pumpWidget(
        WalletAppTestWidget(
          child: const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
            MockSetupSecurityBloc(),
            const SetupSecurityDeviceIncompatibleError(error: 'n/a'),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'device not supported' explanation is shown
      final headlineFinder = find.text(l10n.errorScreenDeviceIncompatibleHeadline);
      final descriptionFinder = find.text(l10n.errorScreenDeviceIncompatibleDescription);
      expect(headlineFinder, findsAtLeastNWidgets(1));
      expect(descriptionFinder, findsOneWidget);

      // Verify the 'close' cta is shown
      final tryAgainCtaFinder = find.text(l10n.generalClose);
      expect(tryAgainCtaFinder, findsOneWidget);
    });
  });
}
