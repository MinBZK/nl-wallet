import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/navigation_service.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/feature/common/widget/button/bottom_back_button.dart';
import 'package:wallet/src/feature/common/widget/button/icon/help_icon_button.dart';
import 'package:wallet/src/feature/qr/scan/bloc/qr_scan_bloc.dart';
import 'package:wallet/src/feature/qr/scan/qr_scan_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

class MockQrScanBloc extends MockBloc<QrScanEvent, QrScanState> implements QrScanBloc {}

void main() {
  setUp(() {
    final dummyChannels = [
      'dev.steenbakker.mobile_scanner/scanner/method',
      'dev.steenbakker.mobile_scanner/scanner/event',
      'dev.steenbakker.mobile_scanner/scanner/deviceOrientation',
    ];
    for (final channel in dummyChannels) {
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
        MethodChannel(channel),
        (MethodCall methodCall) async => null,
      );
    }
  });

  group('goldens', () {
    testGoldens('ltc7 ltc16 ltc19 QrScanInitial', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScanScreen().withState<QrScanBloc, QrScanState>(
          MockQrScanBloc(),
          QrScanInitial(),
        ),
      );
      await screenMatchesGolden('qr_scan_initial');
    });

    testGoldens('ltc7 ltc16 ltc19 QrScanFailure', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScanScreen().withState<QrScanBloc, QrScanState>(
          MockQrScanBloc(),
          QrScanFailure(),
        ),
      );
      await tester.pumpAndSettle();
      await screenMatchesGolden('qr_scan_failure');
    });

    testGoldens('ltc7 ltc16 ltc19 QrScanNoPermission', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScanScreen().withState<QrScanBloc, QrScanState>(
          MockQrScanBloc(),
          const QrScanNoPermission(permanentlyDenied: true),
        ),
      );
      await screenMatchesGolden('qr_scan_no_permission');
    });

    testGoldens('ltc7 ltc16 ltc19 QrScanNoPermission Dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScanScreen().withState<QrScanBloc, QrScanState>(
          MockQrScanBloc(),
          const QrScanNoPermission(permanentlyDenied: true),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('qr_scan_no_permission.dark');
    });

    testGoldens('ltc7 ltc16 ltc19 QrScanNoPermission Landscape', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScanScreen().withState<QrScanBloc, QrScanState>(
          MockQrScanBloc(),
          const QrScanNoPermission(permanentlyDenied: true),
        ),
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('qr_scan_no_permission.landscape');
    });

    testGoldens('ltc7 ltc16 ltc19 QrScanScanning', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScanScreen().withState<QrScanBloc, QrScanState>(
          MockQrScanBloc(),
          QrScanScanning(),
        ),
      );
      await screenMatchesGolden('qr_scan_scanning');
    });

    testGoldens('ltc7 ltc16 ltc19 QrScanSuccess', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScanScreen().withState<QrScanBloc, QrScanState>(
          MockQrScanBloc(),
          const QrScanSuccess(GenericNavigationRequest('/')),
        ),
        providers: [RepositoryProvider<NavigationService>(create: (c) => MockNavigationService())],
      );
      await screenMatchesGolden('qr_scan_success');
    });

    testGoldens('ltc7 ltc16 ltc19 QrScanScanning Dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScanScreen().withState<QrScanBloc, QrScanState>(
          MockQrScanBloc(),
          const QrScanSuccess(GenericNavigationRequest('/')),
        ),
        brightness: Brightness.dark,
        providers: [RepositoryProvider<NavigationService>(create: (c) => MockNavigationService())],
      );
      await screenMatchesGolden('qr_scan_success.dark');
    });

    testGoldens('ltc7 ltc16 ltc19 QrScanLoading', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScanScreen().withState<QrScanBloc, QrScanState>(
          MockQrScanBloc(),
          const QrScanLoading(),
        ),
      );
      await screenMatchesGolden('qr_scan_loading');
    });
  });

  group('widgets', () {
    testWidgets('ltc7 ltc16 ltc19 back button is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScanScreen().withState<QrScanBloc, QrScanState>(
          MockQrScanBloc(),
          const QrScanLoading(),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.generalBottomBackCta), findsOneWidget);
    });

    testWidgets('ltc7 ltc16 ltc19 no permission state shows a single back button and the help button', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScanScreen().withState<QrScanBloc, QrScanState>(
          MockQrScanBloc(),
          const QrScanNoPermission(permanentlyDenied: false),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(find.byType(BottomBackButton), findsNothing);
      expect(find.text(l10n.generalBottomBackCta), findsOneWidget);
      expect(find.byType(HelpIconButton), findsOneWidget);
    });

    testWidgets('ltc7 ltc16 ltc19 navigation is delegated to navigation service', (tester) async {
      final NavigationService mockNavigationService = MockNavigationService();
      await tester.pumpWidgetWithAppWrapper(
        const QrScanScreen()
            .withState<QrScanBloc, QrScanState>(
              MockQrScanBloc(),
              const QrScanSuccess(GenericNavigationRequest('/issuance')),
            )
            .withDependency((context) => mockNavigationService),
      );
      await tester.pumpAndSettle();

      verify(mockNavigationService.handleNavigationRequest(const GenericNavigationRequest('/issuance'))).called(1);
    });
  });
}
