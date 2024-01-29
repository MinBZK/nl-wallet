import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/navigation_service.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/feature/qr/bloc/qr_bloc.dart';
import 'package:wallet/src/feature/qr/qr_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.dart';
import '../../util/device_utils.dart';

class MockQrScanBloc extends MockBloc<QrEvent, QrState> implements QrBloc {}

void main() {
  setUp(() {
    // Mock the scanner
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
        const MethodChannel('dev.steenbakker.mobile_scanner/scanner/method'), (MethodCall methodCall) async {
      return null;
    });
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(
        const MethodChannel('dev.steenbakker.mobile_scanner/scanner/event'), (MethodCall methodCall) async {
      return null;
    });
  });

  group('goldens', () {
    DeviceBuilder deviceBuilder(WidgetTester tester) => DeviceUtils.deviceBuilderWithPrimaryScrollController;

    testGoldens('QrScanInitial', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester)
          ..addScenario(
            widget: const QrScreen().withState<QrBloc, QrState>(
              MockQrScanBloc(),
              QrScanInitial(),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'qr_scan_initial');
    });

    testGoldens('QrScanFailure', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester)
          ..addScenario(
            widget: const QrScreen().withState<QrBloc, QrState>(
              MockQrScanBloc(),
              QrScanFailure(),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'qr_scan_failure');
    });

    testGoldens('QrScanNoPermission', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester)
          ..addScenario(
            widget: const QrScreen().withState<QrBloc, QrState>(
              MockQrScanBloc(),
              const QrScanNoPermission(true),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'qr_scan_no_permission');
    });

    testGoldens('QrScanScanning', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester)
          ..addScenario(
            widget: const QrScreen().withState<QrBloc, QrState>(
              MockQrScanBloc(),
              QrScanScanning(),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'qr_scan_scanning');
    });

    testGoldens('QrScanSuccess', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester)
          ..addScenario(
            widget: const QrScreen().withState<QrBloc, QrState>(
              MockQrScanBloc(),
              const QrScanSuccess(GenericNavigationRequest('/')),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'qr_scan_success');
    });

    testGoldens('QrScanScanning Dark', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester)
          ..addScenario(
            widget: const QrScreen().withState<QrBloc, QrState>(
              MockQrScanBloc(),
              const QrScanSuccess(GenericNavigationRequest('/')),
            ),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'qr_scan_success.dark');
    });

    testGoldens('QrScanLoading', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester)
          ..addScenario(
            widget: const QrScreen().withState<QrBloc, QrState>(
              MockQrScanBloc(),
              const QrScanLoading(),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'qr_scan_loading');
    });

    testGoldens('My code tab', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester)
          ..addScenario(
            widget: const QrScreen().withState<QrBloc, QrState>(
              MockQrScanBloc(),
              QrScanInitial(),
            ),
          ),
        wrapper: walletAppWrapper(),
      );

      // Tap the 'my code' tab on every instance
      const myCodeTabTitle = 'My code';
      for (int i = 0; i < find.text(myCodeTabTitle).evaluate().length; i++) {
        await tester.tap(find.text(myCodeTabTitle).at(i));
      }
      await tester.pumpAndSettle();
      await screenMatchesGolden(tester, 'my_code_tab');
    });

    testGoldens('Scan Explanation sheet', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScreen().withState<QrBloc, QrState>(
          MockQrScanBloc(),
          QrScanFailure(),
        ),
      );
      // Tap the explanation button to open the sheet
      await tester.tap(find.text('How does scanning work?'));
      await tester.pumpAndSettle();
      await screenMatchesGolden(tester, 'scan_explanation_sheet');
    });

    testGoldens('Qr Explanation sheet', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScreen().withState<QrBloc, QrState>(
          MockQrScanBloc(),
          QrScanFailure(),
        ),
      );
      // Navigate to the my code tab
      await tester.tap(find.text('My code'));
      await tester.pumpAndSettle();
      // Tap the explanation button to open the sheet
      await tester.tap(find.text('How does my QR-code work?'));
      await tester.pumpAndSettle();
      await screenMatchesGolden(tester, 'qr_explanation_sheet');
    });
  });

  group('widgets', () {
    testWidgets('back button is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScreen().withState<QrBloc, QrState>(
          MockQrScanBloc(),
          const QrScanLoading(),
        ),
      );

      expect(find.text('Back'), findsOneWidget);
    });

    testWidgets('navigation is delegated to navigation service', (tester) async {
      final NavigationService mockNavigationService = MockNavigationService();
      await tester.pumpWidgetWithAppWrapper(
        const QrScreen()
            .withState<QrBloc, QrState>(
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
