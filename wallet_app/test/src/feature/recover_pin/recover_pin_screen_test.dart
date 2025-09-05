import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/common/widget/button/icon/close_icon_button.dart';
import 'package:wallet/src/feature/recover_pin/bloc/recover_pin_bloc.dart';
import 'package:wallet/src/feature/recover_pin/recover_pin_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';

class MockRecoverPinBloc extends MockBloc<RecoverPinEvent, RecoverPinState> implements RecoverPinBloc {}

void main() {
  const String mockAuthUrl = 'https://mock.auth.url';
  const GenericError mockError = GenericError('test', sourceError: 'test error');
  const NetworkError mockNetworkErrorNoInternet = NetworkError(hasInternet: false, sourceError: 'test error');
  const NetworkError mockNetworkErrorWithInternet = NetworkError(hasInternet: true, sourceError: 'test error');
  const SessionError mockSessionError = SessionError(state: SessionState.expired, sourceError: 'test error');
  const ValidatePinError mockValidatePinError = ValidatePinError(PinValidationError.other, sourceError: 'test error');

  group('goldens', () {
    testGoldens('RecoverPinInitial', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen()
            .withState<RecoverPinBloc, RecoverPinState>(MockRecoverPinBloc(), const RecoverPinInitial()),
      );
      await screenMatchesGolden('initial.light');
    });

    testGoldens('RecoverPinInitial - dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen()
            .withState<RecoverPinBloc, RecoverPinState>(MockRecoverPinBloc(), const RecoverPinInitial()),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('initial.dark');
    });

    testGoldens('RecoverPinLoadingDigidUrl', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen()
            .withState<RecoverPinBloc, RecoverPinState>(MockRecoverPinBloc(), const RecoverPinLoadingDigidUrl()),
      );
      await screenMatchesGolden('loading_digid_url.light');
    });

    testGoldens('RecoverPinAwaitingDigidAuthentication', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen().withState<RecoverPinBloc, RecoverPinState>(
          MockRecoverPinBloc(),
          const RecoverPinAwaitingDigidAuthentication(mockAuthUrl),
        ),
      );
      await screenMatchesGolden('awaiting_digid_authentication.light');
    });

    testGoldens('RecoverPinVerifyingDigidAuthentication', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen().withState<RecoverPinBloc, RecoverPinState>(
          MockRecoverPinBloc(),
          const RecoverPinVerifyingDigidAuthentication(),
        ),
      );
      await screenMatchesGolden('verifying_digid_authentication.light');
    });

    testGoldens('RecoverPinDigidMismatch', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen()
            .withState<RecoverPinBloc, RecoverPinState>(MockRecoverPinBloc(), const RecoverPinDigidMismatch()),
      );
      await screenMatchesGolden('digid_mismatch.light');
    });

    testGoldens('RecoverPinStopped', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen()
            .withState<RecoverPinBloc, RecoverPinState>(MockRecoverPinBloc(), const RecoverPinStopped()),
      );
      await screenMatchesGolden('stopped.light');
    });

    testGoldens('RecoverPinChooseNewPin', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen().withState<RecoverPinBloc, RecoverPinState>(
          MockRecoverPinBloc(),
          const RecoverPinChooseNewPin(authUrl: mockAuthUrl, pin: '1234'),
        ),
      );
      await screenMatchesGolden('choose_new_pin.light');
    });

    testGoldens('RecoverPinConfirmNewPin', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen().withState<RecoverPinBloc, RecoverPinState>(
          MockRecoverPinBloc(),
          const RecoverPinConfirmNewPin(authUrl: mockAuthUrl, selectedPin: '123456', pin: '12', isRetrying: false),
        ),
      );
      await screenMatchesGolden('confirm_new_pin.light');
    });

    testGoldens('RecoverPinConfirmNewPin -> StopSheet', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen().withState<RecoverPinBloc, RecoverPinState>(
          MockRecoverPinBloc(),
          const RecoverPinConfirmNewPin(authUrl: mockAuthUrl, selectedPin: '123456', pin: '12', isRetrying: false),
        ),
        brightness: Brightness.dark,
      );

      // Tap the close button to spawn the stop sheet
      await tester.tap(find.byKey(kCloseIconButtonKey));
      await tester.pumpAndSettle();

      await screenMatchesGolden('confirm_new_pin.stop_sheet.dark');
    });

    testGoldens('RecoverPinConfirmNewPin - Dark - Landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen().withState<RecoverPinBloc, RecoverPinState>(
          MockRecoverPinBloc(),
          const RecoverPinConfirmNewPin(authUrl: mockAuthUrl, selectedPin: '123456', pin: '12345', isRetrying: false),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('confirm_new_pin.dark.landscape');
    });

    testGoldens('RecoverPinUpdatingPin', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen()
            .withState<RecoverPinBloc, RecoverPinState>(MockRecoverPinBloc(), const RecoverPinUpdatingPin()),
      );
      await screenMatchesGolden('updating_pin.light');
    });

    testGoldens('RecoverPinSuccess', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen()
            .withState<RecoverPinBloc, RecoverPinState>(MockRecoverPinBloc(), const RecoverPinSuccess()),
      );
      await screenMatchesGolden('success.light');
    });

    testGoldens('RecoverPinSelectPinFailed', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen().withState<RecoverPinBloc, RecoverPinState>(
          MockRecoverPinBloc(),
          const RecoverPinSelectPinFailed(error: mockValidatePinError),
        ),
      );
      await screenMatchesGolden('select_pin_failed.light');
    });

    testGoldens('RecoverPinConfirmPinFailed', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen().withState<RecoverPinBloc, RecoverPinState>(
          MockRecoverPinBloc(),
          const RecoverPinConfirmPinFailed(error: mockError),
        ),
      );
      await screenMatchesGolden('confirm_pin_failed.light');
    });

    testGoldens('RecoverPinConfirmPinFailed - Retry', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen().withState<RecoverPinBloc, RecoverPinState>(
          MockRecoverPinBloc(),
          const RecoverPinConfirmPinFailed(error: mockError, canRetry: false),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('confirm_pin_failed.retry.dark');
    });

    testGoldens('RecoverPinDigidFailure', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen().withState<RecoverPinBloc, RecoverPinState>(
          MockRecoverPinBloc(),
          const RecoverPinDigidFailure(error: mockError),
        ),
      );
      await screenMatchesGolden('digid_failure.light');
    });

    testGoldens('RecoverPinDigidLoginCancelled', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen()
            .withState<RecoverPinBloc, RecoverPinState>(MockRecoverPinBloc(), const RecoverPinDigidLoginCancelled()),
      );
      await screenMatchesGolden('digid_login_cancelled.light');
    });

    testGoldens('RecoverPinNetworkError (no internet)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen().withState<RecoverPinBloc, RecoverPinState>(
          MockRecoverPinBloc(),
          const RecoverPinNetworkError(error: mockNetworkErrorNoInternet, hasInternet: false),
        ),
      );
      await screenMatchesGolden('network_error.no_internet.light');
    });

    testGoldens('RecoverPinNetworkError (with internet)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen().withState<RecoverPinBloc, RecoverPinState>(
          MockRecoverPinBloc(),
          const RecoverPinNetworkError(error: mockNetworkErrorWithInternet, hasInternet: true),
        ),
      );
      await screenMatchesGolden('network_error.with_internet.light');
    });

    testGoldens('RecoverPinGenericError', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen().withState<RecoverPinBloc, RecoverPinState>(
          MockRecoverPinBloc(),
          const RecoverPinGenericError(error: mockError),
        ),
      );
      await screenMatchesGolden('generic_error.light');
    });

    testGoldens('RecoverPinSessionExpired', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverPinScreen().withState<RecoverPinBloc, RecoverPinState>(
          MockRecoverPinBloc(),
          const RecoverPinSessionExpired(error: mockSessionError),
        ),
      );
      await screenMatchesGolden('session_expired.light');
    });
  });
}
