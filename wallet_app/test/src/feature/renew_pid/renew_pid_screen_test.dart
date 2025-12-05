import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/pid/accept_offered_pid_usecase.dart';
import 'package:wallet/src/feature/common/dialog/stop_digid_login_dialog.dart';
import 'package:wallet/src/feature/renew_pid/bloc/renew_pid_bloc.dart';
import 'package:wallet/src/feature/renew_pid/renew_pid_screen.dart';
import 'package:wallet/src/feature/renew_pid/renew_pid_stop_sheet.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../mocks/wallet_mocks.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

class MockRenewPidBloc extends MockBloc<RenewPidEvent, RenewPidState> implements RenewPidBloc {}

void main() {
  final sampleAttributes = WalletMockData.card.attributes;
  final sampleCards = [WalletMockData.card, WalletMockData.altCard];

  group('goldens', () {
    testGoldens('ltc66 RenewPidInitial', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(MockRenewPidBloc(), const RenewPidInitial()),
      );
      await screenMatchesGolden('initial.light');
    });

    testGoldens('ltc66 RenewPidInitial - dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(MockRenewPidBloc(), const RenewPidInitial()),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('initial.dark');
    });

    testGoldens('ltc66 RenewPidInitial - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(MockRenewPidBloc(), const RenewPidInitial()),
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('initial.landscape.light');
    });

    testGoldens('ltc66 RenewPidInitial - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(MockRenewPidBloc(), const RenewPidInitial()),
        textScaleSize: 1.5,
      );
      await screenMatchesGolden('initial.scaled.light');
    });

    testGoldens('ltc66 RenewPidLoadingDigidUrl', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(
          MockRenewPidBloc(),
          const RenewPidLoadingDigidUrl(),
        ),
      );
      await screenMatchesGolden('loading_digid_url.light');
    });

    testGoldens('ltc66 RenewPidAwaitingDigidAuthentication', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(
          MockRenewPidBloc(),
          const RenewPidAwaitingDigidAuthentication('https://auth_url'),
        ),
      );
      await screenMatchesGolden('awaiting_digid_authentication.light');
    });

    testGoldens('ltc66 RenewPidVerifyingDigidAuthentication', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(
          MockRenewPidBloc(),
          const RenewPidVerifyingDigidAuthentication(),
        ),
      );
      await screenMatchesGolden('verifying_digid_authentication.light');
    });

    testGoldens('ltc66 RenewPidCheckData', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(
          MockRenewPidBloc(),
          RenewPidCheckData(availableAttributes: sampleAttributes),
        ),
      );
      await screenMatchesGolden('check_data.light');
    });

    testGoldens('ltc66 RenewPidConfirmPin', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(
          MockRenewPidBloc(),
          RenewPidConfirmPin(sampleAttributes),
        ),
        providers: [
          RepositoryProvider<AcceptOfferedPidUseCase>(create: (c) => MockAcceptOfferedPidUseCase()),
        ],
      );
      await screenMatchesGolden('confirm_pin.light');
    });

    testGoldens('ltc66 RenewPidUpdatingCards', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(
          MockRenewPidBloc(),
          const RenewPidUpdatingCards(),
        ),
        providers: [
          RepositoryProvider<AcceptOfferedPidUseCase>(create: (c) => MockAcceptOfferedPidUseCase()),
        ],
      );
      await screenMatchesGolden('updating_cards.light');
    });

    testGoldens('ltc66 RenewPidSuccess', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(MockRenewPidBloc(), RenewPidSuccess(sampleCards)),
      );
      await screenMatchesGolden('success.light');
    });

    testGoldens('ltc66 RenewPidDigidMismatch', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(
          MockRenewPidBloc(),
          const RenewPidDigidMismatch(),
        ),
      );
      await screenMatchesGolden('digid_mismatch.light');
    });

    testGoldens('ltc66 RenewPidDigidLoginCancelled', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(
          MockRenewPidBloc(),
          const RenewPidDigidLoginCancelled(),
        ),
      );
      await screenMatchesGolden('digid_login_cancelled.light');
    });

    testGoldens('ltc66 RenewPidStopped', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(MockRenewPidBloc(), const RenewPidStopped()),
      );
      await screenMatchesGolden('stopped.light');
    });

    testGoldens('ltc66 RenewPidNetworkError (No Internet)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(
          MockRenewPidBloc(),
          const RenewPidNetworkError(error: NetworkError(hasInternet: false, sourceError: 'test'), hasInternet: false),
        ),
      );
      await screenMatchesGolden('network_error.no_internet.light');
    });

    testGoldens('ltc66 RenewPidNetworkError (With Internet)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(
          MockRenewPidBloc(),
          const RenewPidNetworkError(error: NetworkError(hasInternet: true, sourceError: 'test'), hasInternet: true),
        ),
      );
      await screenMatchesGolden('network_error.with_internet.light');
    });

    testGoldens('ltc66 RenewPidDigidFailure', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(
          MockRenewPidBloc(),
          const RenewPidDigidFailure(error: GenericError('digid', sourceError: 'test')),
        ),
      );
      await screenMatchesGolden('digid_failure.light');
    });

    testGoldens('ltc66 RenewPidGenericError', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(
          MockRenewPidBloc(),
          const RenewPidGenericError(error: GenericError('generic', sourceError: 'test')),
        ),
      );
      await screenMatchesGolden('generic_error.light');
    });

    testGoldens('ltc66 RenewPidSessionExpired', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(
          MockRenewPidBloc(),
          const RenewPidSessionExpired(
            error: SessionError(state: SessionState.expired, sourceError: 'test'),
          ),
        ),
      );
      await screenMatchesGolden('session_expired.light');
    });
  });

  group('dialogs', () {
    testGoldens('ltc66 shows StopDigidLoginDialog when stopping in AwaitingDigidAuthentication', (tester) async {
      // Set up screen with AwaitingDigidAuthentication state
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(
          MockRenewPidBloc(),
          const RenewPidAwaitingDigidAuthentication('https://auth.url'),
        ),
      );
      await tester.pumpAndSettle();

      // Tap the close button in the app bar (should show StopDigidLoginDialog)
      final l10n = await TestUtils.englishLocalizations;
      final buttonFinder = find.text(l10n.generalStop);
      expect(buttonFinder, findsOneWidget);
      await tester.tap(buttonFinder);
      await tester.pumpAndSettle();

      // Verify the StopDigidLoginDialog is present
      expect(find.byType(StopDigidLoginDialog), findsOneWidget);

      await screenMatchesGolden('stop_digid_login_dialog.light');
    });

    testGoldens('ltc66 shows RenewPidStopSheet when stopping in RenewPidInitial', (tester) async {
      // Set up screen with Initial state
      await tester.pumpWidgetWithAppWrapper(
        const RenewPidScreen().withState<RenewPidBloc, RenewPidState>(
          MockRenewPidBloc(),
          const RenewPidInitial(),
        ),
      );
      await tester.pumpAndSettle();

      // Tap the close button in the app bar (should show RenewPidStopSheet)
      final closeIconFinder = find.byTooltip('Close');
      expect(closeIconFinder, findsOneWidget);
      await tester.tap(closeIconFinder);
      await tester.pumpAndSettle();

      // Verify RenewPidStopSheet is present
      expect(find.byType(RenewPidStopSheet), findsOneWidget);
      await screenMatchesGolden('stop_renew_pid_sheet.light');
    });
  });
}
