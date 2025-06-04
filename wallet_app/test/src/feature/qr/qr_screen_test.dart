import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/navigation_service.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/feature/qr/bloc/qr_bloc.dart';
import 'package:wallet/src/feature/qr/qr_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.dart';
import '../../test_util/golden_utils.dart';

class MockQrScanBloc extends MockBloc<QrEvent, QrState> implements QrBloc {}

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
    testGoldens('QrScanInitial', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScreen().withState<QrBloc, QrState>(
          MockQrScanBloc(),
          QrScanInitial(),
        ),
      );
      await screenMatchesGolden('qr_scan_initial');
    });

    testGoldens('QrScanFailure', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScreen().withState<QrBloc, QrState>(
          MockQrScanBloc(),
          QrScanFailure(),
        ),
      );
      await tester.pumpAndSettle();
      await screenMatchesGolden('qr_scan_failure');
    });

    testGoldens('QrScanNoPermission', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScreen().withState<QrBloc, QrState>(
          MockQrScanBloc(),
          const QrScanNoPermission(permanentlyDenied: true),
        ),
      );
      await screenMatchesGolden('qr_scan_no_permission');
    });

    testGoldens('QrScanScanning', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScreen().withState<QrBloc, QrState>(
          MockQrScanBloc(),
          QrScanScanning(),
        ),
      );
      await screenMatchesGolden('qr_scan_scanning');
    });

    testGoldens('QrScanSuccess', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScreen().withState<QrBloc, QrState>(
          MockQrScanBloc(),
          const QrScanSuccess(GenericNavigationRequest('/')),
        ),
        providers: [RepositoryProvider<NavigationService>(create: (c) => MockNavigationService())],
      );
      await screenMatchesGolden('qr_scan_success');
    });

    testGoldens('QrScanScanning Dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScreen().withState<QrBloc, QrState>(
          MockQrScanBloc(),
          const QrScanSuccess(GenericNavigationRequest('/')),
        ),
        brightness: Brightness.dark,
        providers: [RepositoryProvider<NavigationService>(create: (c) => MockNavigationService())],
      );
      await screenMatchesGolden('qr_scan_success.dark');
    });

    testGoldens('QrScanLoading', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const QrScreen().withState<QrBloc, QrState>(
          MockQrScanBloc(),
          const QrScanLoading(),
        ),
      );
      await screenMatchesGolden('qr_scan_loading');
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
