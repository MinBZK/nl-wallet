import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../bloc/qr_bloc.dart';

class QrScanner extends StatefulWidget {
  const QrScanner({super.key});

  @override
  State<QrScanner> createState() => _QrScannerState();
}

class _QrScannerState extends State<QrScanner> {
  final MobileScannerController cameraController = MobileScannerController(formats: [BarcodeFormat.qrCode]);

  @override
  Widget build(BuildContext context) {
    return MobileScanner(
      controller: cameraController,
      overlay: _buildOverlay(),
      placeholderBuilder: (context, child) => const CenteredLoadingIndicator(),
      errorBuilder: (context, ex, child) {
        Fimber.e('Failed to start camera', ex: ex);
        return Center(child: Text(context.l10n.errorScreenGenericHeadline));
      },
      onDetect: (capture) {
        final event = QrScanCodeDetected(capture.barcodes.first);
        context.read<QrBloc>().add(event);
      },
    );
  }

  Widget _buildOverlay() {
    return Stack(
      children: [
        _buildAlignedScanQrHint(),
        _buildPositionedFlashLightButton(),
      ],
    );
  }

  Widget _buildAlignedScanQrHint() {
    return Align(
      alignment: Alignment.topCenter,
      child: Container(
        width: double.infinity,
        padding: const EdgeInsets.symmetric(vertical: 8),
        margin: EdgeInsets.only(top: context.mediaQuery.padding.top),
        color: context.theme.appBarTheme.backgroundColor?.withOpacity(0.9),
        child: Text(
          context.l10n.qrScreenScanHint,
          textAlign: TextAlign.center,
          style: context.textTheme.bodyLarge,
        ),
      ),
    );
  }

  Widget _buildPositionedFlashLightButton() {
    final buttonRadius = BorderRadius.circular(200);
    return ValueListenableBuilder(
      valueListenable: cameraController.hasTorchState,
      builder: (context, hasTorch, child) {
        if (hasTorch == true && child != null) return child;
        return const SizedBox.shrink();
      },
      child: Positioned(
        bottom: 32,
        left: 0,
        right: 0,
        child: Center(
          child: Material(
            color: Colors.white,
            borderRadius: buttonRadius,
            child: InkWell(
              borderRadius: buttonRadius,
              onTap: () => cameraController.toggleTorch(),
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 16),
                child: ValueListenableBuilder(
                  valueListenable: cameraController.torchState,
                  builder: (context, torch, child) {
                    return Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(
                          torch.isOn ? Icons.flashlight_off_outlined : Icons.flashlight_on_outlined,
                          color: context.colorScheme.onSecondary,
                          size: 16,
                        ),
                        const SizedBox(width: 12),
                        Text(
                          torch.isOn ? context.l10n.qrScreenDisableTorchCta : context.l10n.qrScreenEnableTorchCta,
                          style: context.textTheme.labelLarge?.copyWith(color: context.colorScheme.onSecondary),
                        ),
                      ],
                    );
                  },
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

extension _TorchStateExtension on TorchState {
  bool get isOn => this == TorchState.on;
}
