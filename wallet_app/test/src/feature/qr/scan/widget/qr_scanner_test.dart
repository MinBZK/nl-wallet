import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:mocktail/mocktail.dart' as mocktail;
import 'package:wallet/src/feature/qr/scan/bloc/qr_scan_bloc.dart';
import 'package:wallet/src/feature/qr/scan/widget/qr_scanner.dart';

import '../../../../../wallet_app_test_widget.dart';

class MockQrScanBloc extends MockBloc<QrScanEvent, QrScanState> implements QrScanBloc {}

void main() {
  late QrScanBloc qrScanBloc;

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
    qrScanBloc = MockQrScanBloc();
  });

  testWidgets('QrScanner renders MobileScanner', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      const QrScanner().withState<QrScanBloc, QrScanState>(qrScanBloc, QrScanScanning()),
    );

    expect(find.byType(MobileScanner), findsOneWidget);
  });

  testWidgets('QrScanner adds QrScanCodeDetected event when a QR code is detected', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      const QrScanner().withState<QrScanBloc, QrScanState>(qrScanBloc, QrScanScanning()),
    );

    // Find the MobileScanner widget
    final mobileScannerFinder = find.byType(MobileScanner);
    expect(mobileScannerFinder, findsOneWidget);

    // Get the widget and call onDetect
    final mobileScanner = tester.widget<MobileScanner>(mobileScannerFinder);

    final barcode = const Barcode(rawValue: 'https://example.com', format: BarcodeFormat.qrCode);
    final capture = BarcodeCapture(barcodes: [barcode]);

    mobileScanner.onDetect!(capture);

    mocktail.verify(() => qrScanBloc.add(QrScanCodeDetected(barcode))).called(1);
  });
}
