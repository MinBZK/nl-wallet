import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/biometrics/get_available_biometrics_usecase.dart';
import 'package:wallet/src/feature/setup_security/bloc/setup_security_bloc.dart';
import 'package:wallet/src/feature/setup_security/setup_security_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

class MockSetupSecurityBloc extends MockBloc<SetupSecurityEvent, SetupSecurityState> implements SetupSecurityBloc {}

void main() {
  group('goldens', () {
    testGoldens('SetupSecuritySelectPinInProgress light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          const SetupSecuritySelectPinInProgress(3),
        ),
      );
      await screenMatchesGolden('in_progress.light');
    });

    testGoldens('SetupSecuritySelectPinFailed (sequentialDigits) light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          const SetupSecuritySelectPinFailed(reason: PinValidationError.tooFewUniqueDigits),
        ),
      );
      await screenMatchesGolden('error.light');
    });

    testGoldens('SetupSecurityPinConfirmationFailed retry NOT allowed light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          const SetupSecurityPinConfirmationFailed(retryAllowed: false),
        ),
      );
      await screenMatchesGolden('pin_confirmation_failed.light');
    });

    testGoldens('SetupSecurityPinConfirmationInProgress light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          const SetupSecurityPinConfirmationInProgress(6),
        ),
      );
      await screenMatchesGolden('pin_confirmation_in_progress.light');
    });

    testGoldens('SetupSecurityConfigureBiometrics fingerOnly light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          const SetupSecurityConfigureBiometrics(biometrics: Biometrics.fingerprint),
        ),
      );
      await screenMatchesGolden('biometrics.finger.light');
    });

    testGoldens('SetupSecurityConfigureBiometrics faceOnly light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          const SetupSecurityConfigureBiometrics(biometrics: Biometrics.face),
        ),
      );
      await screenMatchesGolden('biometrics.face.light');
    });

    testGoldens('SetupSecurityConfigureBiometrics some dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          const SetupSecurityConfigureBiometrics(biometrics: Biometrics.some),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('biometrics.some.dark');
    });

    testGoldens('SetupSecurityCompleted light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          const SetupSecurityCompleted(),
        ),
      );
      await screenMatchesGolden('completed.light');
    });

    testGoldens('SetupSecurityPinConfirmationFailed retry NOT allowed dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          const SetupSecurityPinConfirmationFailed(retryAllowed: false),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('pin_confirmation_failed.dark');
    });

    testGoldens('SetupSecurityCreatingWallet light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          SetupSecurityCreatingWallet(),
        ),
      );
      await screenMatchesGolden('creating_wallet.light');
    });

    testGoldens('SetupSecuritySelectPinFailed (tooFewUniqueDigits) dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          const SetupSecuritySelectPinFailed(reason: PinValidationError.tooFewUniqueDigits),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('error.dark');
    });
  });

  group('widgets', () {
    testWidgets('SetupSecurityScreen shows the correct title for SetupSecuritySelectPinInProgress state',
        (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          const SetupSecuritySelectPinInProgress(0),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the title is shown
      final titleFinder = find.text(l10n.setupSecuritySelectPinPageTitle, findRichText: true);
      expect(titleFinder, findsOneWidget);
    });

    testWidgets('SetupSecurityScreen shows the correct title for SetupSecurityPinConfirmationInProgress state',
        (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          const SetupSecurityPinConfirmationInProgress(0),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the title is shown
      final titleFinder = find.text(l10n.setupSecurityConfirmationPageTitle, findRichText: true);
      expect(titleFinder, findsOneWidget);
    });

    testWidgets('SetupSecurityScreen shows the no internet error for SetupSecurityNetworkError(hasInternet=false)',
        (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          const SetupSecurityNetworkError(
            hasInternet: false,
            error: NetworkError(hasInternet: false, sourceError: 'test'),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'no internet' title is shown
      final noInternetHeadlineFinder = find.text(l10n.errorScreenNoInternetHeadline, findRichText: true);
      expect(noInternetHeadlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'try again' cta is shown
      final tryAgainCtaFinder = find.text(l10n.generalRetry, findRichText: true);
      expect(tryAgainCtaFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta, findRichText: true);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('SetupSecurityScreen shows the server error for SetupSecurityNetworkError(hasInternet=true)',
        (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          const SetupSecurityNetworkError(
            hasInternet: true,
            error: NetworkError(hasInternet: true, sourceError: 'test'),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'server error' title is shown
      final noInternetHeadlineFinder = find.text(l10n.errorScreenServerHeadline, findRichText: true);
      expect(noInternetHeadlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'try again' cta is shown
      final tryAgainCtaFinder = find.text(l10n.generalRetry, findRichText: true);
      expect(tryAgainCtaFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta, findRichText: true);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('SetupSecurityScreen shows the generic error for SetupSecurityGenericError state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          const SetupSecurityGenericError(error: GenericError('generic', sourceError: 'test')),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'something went wrong' title is shown
      final headlineFinder = find.text(l10n.errorScreenGenericHeadline, findRichText: true);
      expect(headlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'try again' cta is shown
      final tryAgainCtaFinder = find.text(l10n.generalRetry, findRichText: true);
      expect(tryAgainCtaFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta, findRichText: true);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('SetupSecurityScreen shows the device incompatible error for SetupSecurityDeviceIncompatibleError',
        (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const SetupSecurityScreen().withState<SetupSecurityBloc, SetupSecurityState>(
          MockSetupSecurityBloc(),
          const SetupSecurityDeviceIncompatibleError(error: HardwareUnsupportedError(sourceError: 'test')),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'device not supported' explanation is shown
      final headlineFinder = find.text(l10n.errorScreenDeviceIncompatibleHeadline, findRichText: true);
      final descriptionFinder = find.text(l10n.errorScreenDeviceIncompatibleDescription, findRichText: true);
      expect(headlineFinder, findsAtLeastNWidgets(1));
      expect(descriptionFinder, findsOneWidget);

      // Verify the 'close' cta is shown
      final tryAgainCtaFinder = find.text(l10n.generalClose);
      expect(tryAgainCtaFinder, findsOneWidget);
    });
  });
}
