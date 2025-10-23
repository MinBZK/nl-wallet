import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/wallet/observe_wallet_locked_usecase.dart';
import 'package:wallet/src/feature/wallet_transfer_target/bloc/wallet_transfer_target_bloc.dart';
import 'package:wallet/src/feature/wallet_transfer_target/wallet_transfer_target_screen.dart';
import 'package:wallet/src/wallet_assets.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.dart';
import '../../test_util/golden_utils.dart';

class MockWalletTransferTargetBloc extends MockBloc<WalletTransferTargetEvent, WalletTransferTargetState>
    implements WalletTransferTargetBloc {}

const _testQrData =
    'Lorem ipsum dolor sit amet, consectetur adipiscing elit, '
    'sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.';

void main() {
  group('goldens', () {
    testGoldens('WalletTransferIntroduction', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferIntroduction(),
        ),
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_introduction.light');
    });

    testGoldens('WalletTransferIntroduction - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferIntroduction(),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_introduction.dark.landscape');
    });

    testGoldens('WalletTransferIntroduction - opt out dialog', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferIntroduction(),
        ),
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );

      // Tap the opt out button
      await tester.tap(find.textContaining('No, create a new'));
      await tester.pumpAndSettle();

      await screenMatchesGolden('wallet_transfer_introduction.opt_out_dialog.light');
    });

    testGoldens('WalletTransferLoadingQrData', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferLoadingQrData(),
        ),
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_loading_qr_data.light');
    });

    testGoldens('WalletTransferLoadingQrData - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferLoadingQrData(),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_loading_qr_data.dark.landscape');
    });

    testGoldens('WalletTransferAwaitingQrScan', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferAwaitingQrScan(_testQrData),
        ),
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await _preCacheWalletLogo(tester);
      await screenMatchesGolden('wallet_transfer_awaiting_qr_scan.light');
    });

    testGoldens('WalletTransferAwaitingQrScan - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferAwaitingQrScan(_testQrData),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await _preCacheWalletLogo(tester);
      await screenMatchesGolden('wallet_transfer_awaiting_qr_scan.dark.landscape');
    });

    testGoldens('WalletTransferAwaitingQrScan - dialog', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferAwaitingQrScan(_testQrData),
        ),
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await _preCacheWalletLogo(tester);

      // Tap the show dialog button
      await tester.tap(find.textContaining('Center QR'));
      await tester.pumpAndSettle();

      await screenMatchesGolden('wallet_transfer_awaiting_qr_scan.dialog.light');
    });

    testGoldens('WalletTransferAwaitingQrScan - dialog - tablet', (tester) async {
      // Extra test on tablet form factor to verify size/orientation based QR widget scaling.
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferAwaitingQrScan(_testQrData),
        ),
        surfaceSize: const Size(834, 1194),
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await _preCacheWalletLogo(tester);

      // Tap the show dialog button
      await tester.tap(find.textContaining('Center QR'));
      await tester.pumpAndSettle();

      await screenMatchesGolden('wallet_transfer_awaiting_qr_scan.dialog.light.tablet');
    });

    testGoldens('WalletTransferAwaitingQrScan - dialog - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferAwaitingQrScan(_testQrData),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await _preCacheWalletLogo(tester);

      // Tap the show dialog button
      await tester.tap(find.textContaining('Center QR'));
      await tester.pumpAndSettle();

      await screenMatchesGolden('wallet_transfer_awaiting_qr_scan.dialog.dark.landscape');
    });

    testGoldens('WalletTransferAwaitingConfirmation', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferAwaitingConfirmation(),
        ),
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_awaiting_confirmation.light');
    });

    testGoldens('WalletTransferAwaitingConfirmation - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferAwaitingConfirmation(),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_awaiting_confirmation.dark.landscape');
    });

    testGoldens('WalletTransferTransferring', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferTransferring(),
        ),
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_transferring.light');
    });

    testGoldens('WalletTransferTransferring - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferTransferring(),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_transferring.dark.landscape');
    });

    testGoldens('WalletTransferTransferring - stop sheet', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferTransferring(),
        ),
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      // Tap the stop button in the AppBar
      await tester.tap(find.byIcon(Icons.block_flipped).first);
      await tester.pumpAndSettle();
      await screenMatchesGolden('wallet_transfer_transferring_stop_sheet.light');
    });

    testGoldens('WalletTransferTransferring - stop sheet - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferTransferring(),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      // Tap the stop button in the AppBar
      await tester.tap(find.byIcon(Icons.block_flipped).first);
      await tester.pumpAndSettle();
      await screenMatchesGolden('wallet_transfer_transferring_stop_sheet.dark.landscape');
    });

    testGoldens('WalletTransferSuccess', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferSuccess(),
        ),
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_success.light');
    });

    testGoldens('WalletTransferSuccess - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferSuccess(),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_success.dark.landscape');
    });

    testGoldens('WalletTransferFailed', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferFailed(GenericError('failed', sourceError: 'failedError')),
        ),
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_failed.light');
    });

    testGoldens('WalletTransferFailed - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferFailed(GenericError('failed', sourceError: 'failedError')),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_failed.dark.landscape');
    });

    testGoldens('WalletTransferStopped', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferStopped(),
        ),
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_stopped.light');
    });

    testGoldens('WalletTransferStopped - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferStopped(),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_stopped.dark.landscape');
    });

    testGoldens('WalletTransferGenericError', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferGenericError(GenericError('generic_error', sourceError: 'mockError')),
        ),
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_generic_error.light');
    });

    testGoldens('WalletTransferGenericError - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferGenericError(GenericError('generic_error', sourceError: 'mockError')),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_generic_error.dark.landscape');
    });

    testGoldens('WalletTransferSessionExpired', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferSessionExpired(SessionError(state: SessionState.expired, sourceError: 'sessionError')),
        ),
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_session_expired.light');
    });

    testGoldens('WalletTransferSessionExpired - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferTargetScreen().withState<WalletTransferTargetBloc, WalletTransferTargetState>(
          MockWalletTransferTargetBloc(),
          const WalletTransferSessionExpired(SessionError(state: SessionState.expired, sourceError: 'sessionError')),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        providers: [RepositoryProvider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase())],
      );
      await screenMatchesGolden('wallet_transfer_session_expired.dark.landscape');
    });
  });
}

/// Helper method to pre-cache the wallet logo asset. Needed to make sure the QrCode
/// is able to render the embedded wallet logo in golden tests.
Future<void> _preCacheWalletLogo(WidgetTester tester) async {
  final context = tester.element(find.byType(QrImageView));
  await tester.runAsync(() async {
    await precacheImage(const AssetImage(WalletAssets.logo_wallet), context);
  });
  await tester.pumpAndSettle();
}
