import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

import '../tab/qr_scan/bloc/qr_scan_bloc.dart';
import 'flashlight_state_proxy.dart';
import 'qr_scanner_frame.dart';

class QrScanner extends StatelessWidget {
  final MobileScannerController cameraController = MobileScannerController(formats: [BarcodeFormat.qrCode]);

  QrScanner({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return QrScannerFrame(
      child: FlashlightStateProxy(
        controller: cameraController,
        child: MobileScanner(
          controller: cameraController,
          allowDuplicates: false,
          onDetect: (Barcode barcode, MobileScannerArguments? args) {
            context.read<QrScanBloc>().add(QrScanCodeDetected(barcode));
          },
        ),
      ),
    );
  }
}
