import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/qr/present/bloc/qr_present_bloc.dart';
import 'package:wallet/src/feature/qr/present/qr_present_screen.dart';
import 'package:wallet/src/util/extension/build_context_extension.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

class MockQrPresentBloc extends MockBloc<QrPresentEvent, QrPresentState> implements QrPresentBloc {}

void main() {
  group('QrPresentScreen goldens', () {
    testGoldens('QrPresentInitial', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrPresentScreen().withState<QrPresentBloc, QrPresentState>(
          MockQrPresentBloc(),
          const QrPresentInitial(),
        ),
      );
      await screenMatchesGolden('qr_present_initial.light');
    });

    testGoldens('QrPresentServerStarted', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrPresentScreen().withState<QrPresentBloc, QrPresentState>(
          MockQrPresentBloc(),
          const QrPresentServerStarted('https://example.org/qr'),
        ),
      );
      await TestUtils.preCacheWalletLogoForQrImageView(tester);
      await screenMatchesGolden('qr_present_server_started.light');
    });

    testGoldens('QrPresentServerStarted - dark - landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrPresentScreen().withState<QrPresentBloc, QrPresentState>(
          MockQrPresentBloc(),
          const QrPresentServerStarted('https://example.org/qr'),
        ),
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await TestUtils.preCacheWalletLogoForQrImageView(tester);
      await screenMatchesGolden('qr_present_server_started.dark.landscape');
    });

    testGoldens('QrPresentConnecting', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrPresentScreen().withState<QrPresentBloc, QrPresentState>(
          MockQrPresentBloc(),
          const QrPresentConnecting(),
        ),
      );
      await screenMatchesGolden('qr_present_connecting.light');
    });

    testGoldens('QrPresentConnected', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrPresentScreen().withState<QrPresentBloc, QrPresentState>(
          MockQrPresentBloc(),
          const QrPresentConnected(deviceRequestReceived: false),
        ),
      );
      await screenMatchesGolden('qr_present_connected.light');
    });

    testGoldens('QrPresentConnectionFailed', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrPresentScreen().withState<QrPresentBloc, QrPresentState>(
          MockQrPresentBloc(),
          const QrPresentConnectionFailed(),
        ),
      );
      await screenMatchesGolden('qr_present_connection_failed.light');
    });

    testGoldens('QrPresentError', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrPresentScreen().withState<QrPresentBloc, QrPresentState>(
          MockQrPresentBloc(),
          const QrPresentError(GenericError('Something went wrong', sourceError: 'error')),
        ),
      );
      await screenMatchesGolden('qr_present_error.light');
    });

    testGoldens('QrPresentBluetoothDisabled - Android', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrPresentScreen().withState<QrPresentBloc, QrPresentState>(
          MockQrPresentBloc(),
          const QrPresentBluetoothDisabled(),
        ),
      );
      await screenMatchesGolden('qr_present_bluetooth_disabled.android.light');
    });

    testGoldens('QrPresentBluetoothDisabled - iOS (dark)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            return Theme(
              data: context.theme.copyWith(platform: .iOS),
              child: const QrPresentScreen().withState<QrPresentBloc, QrPresentState>(
                MockQrPresentBloc(),
                const QrPresentBluetoothDisabled(),
              ),
            );
          },
        ),
        brightness: .dark,
      );
      await screenMatchesGolden('qr_present_bluetooth_disabled.ios.dark');
    });

    group('QrPresentScreen - Centered QR Dialog', () {
      testWidgets('Center qr button shows qr dialog', (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const QrPresentScreen().withState<QrPresentBloc, QrPresentState>(
            MockQrPresentBloc(),
            const QrPresentServerStarted('https://example.org/qr'),
          ),
        );
        await TestUtils.preCacheWalletLogoForQrImageView(tester);
        final l10n = await TestUtils.englishLocalizations;
        await tester.tap(find.text(l10n.qrPresentScreenCenterQrCodeCta));
        await tester.pumpAndSettle();

        expect(find.text(l10n.qrPresentScreenDialogTitle), findsOneWidget);
      });
    });
  });
}
