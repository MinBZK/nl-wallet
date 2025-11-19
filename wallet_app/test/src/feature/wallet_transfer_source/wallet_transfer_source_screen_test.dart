import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/configuration/configuration_repository.dart';
import 'package:wallet/src/domain/model/configuration/flutter_app_configuration.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/pin/check_pin_usecase.dart';
import 'package:wallet/src/domain/usecase/transfer/confirm_wallet_transfer_usecase.dart';
import 'package:wallet/src/domain/usecase/version/get_version_string_usecase.dart';
import 'package:wallet/src/feature/wallet_transfer_source/bloc/wallet_transfer_source_bloc.dart';
import 'package:wallet/src/feature/wallet_transfer_source/wallet_transfer_source_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.dart';
import '../../test_util/golden_utils.dart';

class MockWalletTransferSourceBloc extends MockBloc<WalletTransferSourceEvent, WalletTransferSourceState>
    implements WalletTransferSourceBloc {}

void main() {
  group('goldens', () {
    testGoldens('WalletTransferInitial', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferInitial(),
        ),
      );
      await screenMatchesGolden('wallet_transfer_initial.light');
    });
    testGoldens('WalletTransferInitial - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferInitial(),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('wallet_transfer_initial.dark.landscape');
    });
    testGoldens('WalletTransferLoading', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferLoading(),
        ),
      );
      await screenMatchesGolden('wallet_transfer_loading.light');
    });
    testGoldens('WalletTransferCancelling', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferCancelling(),
        ),
      );
      await screenMatchesGolden('wallet_transfer_cancelling.light');
    });
    testGoldens('WalletTransferIntroduction', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferIntroduction(),
        ),
      );
      await screenMatchesGolden('wallet_transfer_introduction.light');
    });
    testGoldens('WalletTransferConfirmPin', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferConfirmPin(),
        ),
        providers: [
          RepositoryProvider<ConfirmWalletTransferUseCase>(create: (c) => MockConfirmWalletTransferUseCase()),
        ],
      );
      await screenMatchesGolden('wallet_transfer_confirm_pin.light');
    });
    testGoldens('WalletTransferTransferring', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferTransferring(),
        ),
        providers: [
          RepositoryProvider<ConfirmWalletTransferUseCase>(create: (c) => MockConfirmWalletTransferUseCase()),
        ],
      );
      await screenMatchesGolden('wallet_transfer_transferring.light');
    });
    testGoldens('WalletTransferSuccess', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferSuccess(),
        ),
      );
      await screenMatchesGolden('wallet_transfer_success.light');
    });
    testGoldens('WalletTransferStopped', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferStopped(),
        ),
      );
      await screenMatchesGolden('wallet_transfer_stopped.light');
    });
    testGoldens('WalletTransferGenericError', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferGenericError(GenericError('generic_error', sourceError: 'mockError')),
        ),
      );
      await screenMatchesGolden('wallet_transfer_generic_error.light');
    });
    testGoldens('WalletTransferNetworkError', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferNetworkError(NetworkError(hasInternet: false, sourceError: 'mockNetworkError')),
        ),
      );
      await screenMatchesGolden('wallet_transfer_network_error.light');
    });
    testGoldens('WalletTransferSessionExpired', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferSessionExpired(SessionError(state: SessionState.expired, sourceError: 'sessionError')),
        ),
      );
      await screenMatchesGolden('wallet_transfer_session_expired.light');
    });
    testGoldens('WalletTransferFailed', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferFailed(GenericError('failed', sourceError: 'failedError')),
        ),
      );
      await screenMatchesGolden('wallet_transfer_failed.light');
    });

    testGoldens('WalletTransferFailed - See Details', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferFailed(GenericError('failed', sourceError: 'failedError')),
        ),
        providers: [
          RepositoryProvider<ConfigurationRepository>(
            create: (c) {
              final mock = MockConfigurationRepository();
              when(mock.appConfiguration).thenAnswer(
                (_) => Stream.value(
                  const FlutterAppConfiguration(
                    backgroundLockTimeout: Duration.zero,
                    idleLockTimeout: Duration.zero,
                    idleWarningTimeout: Duration.zero,
                    staticAssetsBaseUrl: 'https://example.org/',
                    pidAttestationTypes: ['com.example.attestationType'],
                    version: 1337,
                  ),
                ),
              );
              return mock;
            },
          ),
          RepositoryProvider<GetVersionStringUseCase>(
            create: (c) {
              final mock = MockGetVersionStringUseCase();
              when(mock.invoke()).thenAnswer((_) async => const Result.success('0.1.2'));
              return mock;
            },
          ),
        ],
      );
      await tester.tap(find.text('See details'));
      await tester.pumpAndSettle();
      await screenMatchesGolden('wallet_transfer_failed.detail_sheet.light');
    });

    testGoldens('WalletTransferLoading - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferLoading(),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('wallet_transfer_loading.dark.landscape');
    });
    testGoldens('WalletTransferIntroduction - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferIntroduction(),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('wallet_transfer_introduction.dark.landscape');
    });
    testGoldens('WalletTransferConfirmPin - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferConfirmPin(),
        ),
        providers: [
          RepositoryProvider<ConfirmWalletTransferUseCase>(create: (c) => MockConfirmWalletTransferUseCase()),
        ],
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('wallet_transfer_confirm_pin.dark.landscape');
    });
    testGoldens('WalletTransferTransferring - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferTransferring(),
        ),
        providers: [
          RepositoryProvider<ConfirmWalletTransferUseCase>(create: (c) => MockConfirmWalletTransferUseCase()),
        ],
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('wallet_transfer_transferring.dark.landscape');
    });
    testGoldens('WalletTransferSuccess - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferSuccess(),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('wallet_transfer_success.dark.landscape');
    });
    testGoldens('WalletTransferStopped - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferStopped(),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('wallet_transfer_stopped.dark.landscape');
    });
    testGoldens('WalletTransferGenericError - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferGenericError(GenericError('generic_error', sourceError: 'mockError')),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('wallet_transfer_generic_error.dark.landscape');
    });
    testGoldens('WalletTransferNetworkError - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferNetworkError(NetworkError(hasInternet: false, sourceError: 'mockNetworkError')),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('wallet_transfer_network_error.dark.landscape');
    });
    testGoldens('WalletTransferSessionExpired - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferSessionExpired(SessionError(state: SessionState.expired, sourceError: 'sessionError')),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('wallet_transfer_session_expired.dark.landscape');
    });
    testGoldens('WalletTransferFailed - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
          MockWalletTransferSourceBloc(),
          const WalletTransferFailed(GenericError('failed', sourceError: 'failedError')),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('wallet_transfer_failed.dark.landscape');
    });
  });

  testGoldens('WalletTransferTransferring - Stop Sheet', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      const WalletTransferSourceScreen().withState<WalletTransferSourceBloc, WalletTransferSourceState>(
        MockWalletTransferSourceBloc(),
        const WalletTransferTransferring(),
      ),
      providers: [RepositoryProvider<ConfirmWalletTransferUseCase>(create: (c) => MockConfirmWalletTransferUseCase())],
    );

    await tester.tap(find.text('Stop'));
    await tester.pumpAndSettle();

    await screenMatchesGolden('wallet_transfer_transferring.stop_sheet.light');
  });
}
