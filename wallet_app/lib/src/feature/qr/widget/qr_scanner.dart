import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../bloc/qr_bloc.dart';
import 'flashlight_button.dart';
import 'qr_scanner_active_announcer.dart';

const kAndroidCameraResolution = Size(960, 1280);

class QrScanner extends StatefulWidget {
  const QrScanner({super.key});

  @override
  State<QrScanner> createState() => _QrScannerState();
}

class _QrScannerState extends State<QrScanner> {
  final Key _scannerKey = GlobalKey();

  late MobileScannerController cameraController;

  @override
  void initState() {
    super.initState();
    cameraController = MobileScannerController(
      formats: [BarcodeFormat.qrCode],
      cameraResolution: kAndroidCameraResolution /* ignored on iOS */,
    );
  }

  @override
  void dispose() {
    cameraController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MobileScanner(
      key: _scannerKey,
      controller: cameraController,
      overlayBuilder: (context, constraints) => _buildOverlay(context),
      placeholderBuilder: (context) => const CenteredLoadingIndicator(),
      errorBuilder: (context, ex) {
        Fimber.e('Failed to start camera', ex: ex);
        return Center(
          child: Text.rich(
            context.l10n.errorScreenGenericHeadline.toTextSpan(context),
            textAlign: TextAlign.center,
          ),
        );
      },
      onDetect: (capture) {
        final event = QrScanCodeDetected(capture.barcodes.first);
        if (this.context.mounted) this.context.read<QrBloc>().add(event);
      },
    );
  }

  Widget _buildOverlay(BuildContext context) {
    return Stack(
      children: [
        _buildAlignedScanQrHint(),
        _buildPositionedFlashLightButton(),
        const QrScannerActiveAnnouncer(),
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
        color: context.theme.appBarTheme.backgroundColor?.withValues(alpha: 0.9),
        child: Text.rich(
          context.l10n.qrScreenScanHint.toTextSpan(context),
          textAlign: TextAlign.center,
          style: context.textTheme.bodyLarge,
        ),
      ),
    );
  }

  Widget _buildPositionedFlashLightButton() {
    if (cameraController.value.torchState == TorchState.unavailable) return const SizedBox.shrink();
    return Positioned(
      bottom: 32,
      left: 0,
      right: 0,
      child: Center(
        child: IntrinsicWidth(
          child: FlashlightButton(
            isOn: cameraController.value.torchState.isOn,
            onPressed: () => _toggleFlashLight(context),
          ),
        ),
      ),
    );
  }

  void _toggleFlashLight(BuildContext context) {
    final l10n = context.l10n;
    final currentOnState = cameraController.value.torchState.isOn;
    final postToggleOnState = !currentOnState;
    cameraController.toggleTorch().then((value) async {
      if (postToggleOnState) {
        await SemanticsService.announce(l10n.flashlightEnabledWCAGAnnouncement, TextDirection.ltr);
      } else {
        await SemanticsService.announce(l10n.flashlightDisabledWCAGAnnouncement, TextDirection.ltr);
      }
    });
  }
}

extension _TorchStateExtension on TorchState {
  bool get isOn => this == TorchState.on;
}
