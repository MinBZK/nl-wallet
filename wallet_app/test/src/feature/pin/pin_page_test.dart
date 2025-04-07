import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/pin/pin_page.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

class MockPinBloc extends MockBloc<PinEvent, PinState> implements PinBloc {}

void main() {
  group('goldens', () {
    testGoldens('PinEntryInProgress - 0 - Initial', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PinPage(onPinValidated: (_) {}).withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinEntryInProgress(0),
        ),
      );
      await tester.pumpAndSettle();
      await screenMatchesGolden('pin_page/pin_initial');
    });

    testGoldens('PinEntryInProgress - 3', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PinPage(onPinValidated: (_) {}).withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinEntryInProgress(3),
        ),
      );
      await tester.pumpAndSettle();
      await screenMatchesGolden('pin_page/pin_entry_in_progress');
    });

    testGoldens('PinValidateInProgress', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PinPage(onPinValidated: (_) {}).withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinValidateInProgress(),
        ),
      );
      await screenMatchesGolden('pin_page/pin_validating');
    });

    testGoldens('PinValidateFailure', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PinPage(onPinValidated: (_) {}).withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinValidateFailure(attemptsLeftInRound: 3, isFinalRound: false),
        ),
      );
      await screenMatchesGolden('pin_page/pin_validate_failure');
    });

    testGoldens('PinValidateFailure - final attempt', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PinPage(onPinValidated: (_) {}).withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinValidateFailure(attemptsLeftInRound: 1, isFinalRound: true),
        ),
      );
      await screenMatchesGolden('pin_page/pin_validate_final_chance');
    });

    testGoldens('PinValidateGenericError', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PinPage(onPinValidated: (_) {}).withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinValidateGenericError(error: GenericError('', sourceError: 'test')),
        ),
      );
      await tester.pumpAndSettle();
      await screenMatchesGolden('pin_page/pin_validate_generic_error');
    });

    testGoldens('PinValidateNetworkError', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PinPage(onPinValidated: (_) {}).withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinValidateNetworkError(
            error: NetworkError(hasInternet: true, sourceError: 'test'),
            hasInternet: true,
          ),
        ),
      );
      await tester.pumpAndSettle();
      await screenMatchesGolden('pin_page/pin_validate_network_error');
    });
  });

  group('widgets', () {
    testWidgets('PinPage renders the correct header', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        PinPage(onPinValidated: (_) {}).withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinEntryInProgress(3),
        ),
      );

      // Wait for widget to settle (triggered by announcement delay)
      await tester.pumpAndSettle();

      // Setup finders
      final headerFinder = find.text(l10n.pinScreenHeader, findRichText: true);

      // Verify all expected widgets show up once
      expect(headerFinder, findsOneWidget);
    });

    testWidgets('PinPage renders the correct header in portrait', (tester) async {
      tester.view.physicalSize = tester.view.physicalSize.flipped;
      addTearDown(() => tester.view.resetPhysicalSize());

      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        PinPage(onPinValidated: (_) {}).withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinEntryInProgress(3),
        ),
      );

      // Wait for widget to settle (triggered by announcement delay)
      await tester.pumpAndSettle();

      // Setup finders
      final headerFinder = find.text(l10n.pinScreenHeader, findRichText: true);

      // Verify all expected widgets show up once
      expect(headerFinder, findsOneWidget);
    });

    testWidgets('PinPage renders the default error with the correct amount of leftover attempts', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        PinPage(onPinValidated: (_) {}).withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinValidateFailure(attemptsLeftInRound: 2, isFinalRound: false),
        ),
      );

      // Wait for the dialog to be shown
      await tester.pumpAndSettle();

      // Setup finders
      final headerFinder = find.text(l10n.pinErrorDialogTitle);
      final attemptsLeftFinder = find.text(l10n.pinErrorDialogNonFinalRoundNonFinalAttempt(2));

      // Verify all expected widgets show up once
      expect(headerFinder, findsOneWidget);
      expect(attemptsLeftFinder, findsOneWidget);
    });

    testWidgets('PinPage executes navigation when blocked', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PinPage(onPinValidated: (_) {}).withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinValidateBlocked(),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text(WalletRoutes.pinBlockedRoute), findsOneWidget);
    });

    testWidgets('PinPage executes navigation when timeout is triggered', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PinPage(onPinValidated: (_) {}).withState<PinBloc, PinState>(
          MockPinBloc(),
          PinValidateTimeout(DateTime.now().add(const Duration(hours: 3))),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text(WalletRoutes.pinTimeoutRoute), findsOneWidget);
    });
  });
}
