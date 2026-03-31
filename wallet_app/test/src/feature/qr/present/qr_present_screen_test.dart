import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/cupertino.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/qr/present/bloc/qr_present_bloc.dart';
import 'package:wallet/src/feature/qr/present/qr_present_screen.dart';

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
  });
}
