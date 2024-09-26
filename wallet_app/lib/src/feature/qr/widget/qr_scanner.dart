import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:native_device_orientation/native_device_orientation.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../bloc/qr_bloc.dart';
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
    return NativeDeviceOrientationReader(
      builder: (context) {
        final int quarterTurns = _resolveScannerTurns(context);
        return RotatedBox(
          quarterTurns: quarterTurns /* rotate camera feed to match device rotation */,
          child: MobileScanner(
            key: _scannerKey,
            controller: cameraController,
            overlayBuilder: (context, constraints) => RotatedBox(
              quarterTurns: quarterTurns * -1 /* revert rotation for nested widget */,
              child: _buildOverlay(context),
            ),
            placeholderBuilder: (context, child) => const CenteredLoadingIndicator(),
            errorBuilder: (context, ex, child) {
              Fimber.e('Failed to start camera', ex: ex);
              return RotatedBox(
                quarterTurns: quarterTurns * -1 /* revert rotation for nested widget */,
                child: Center(
                  child: Text.rich(
                    context.l10n.errorScreenGenericHeadline.toTextSpan(context),
                  ),
                ),
              );
            },
            onDetect: (capture) {
              final event = QrScanCodeDetected(capture.barcodes.first);
              if (this.context.mounted) this.context.read<QrBloc>().add(event);
            },
          ),
        );
      },
    );
  }

  /// Resolve the quarter turns that should be applied to the [MobileScanner], since the image feed provided by
  /// the plugin is always rendered in portrait (even when the device is rotated). By using the quarter turns provided
  /// by this plugin the image feed will be rendered correctly. Note that this should NOT be applied to any other
  /// widgets, as that is resolved by the framework as normal.
  int _resolveScannerTurns(BuildContext context) {
    final orientation = NativeDeviceOrientationReader.orientation(context);
    return switch (orientation) {
      NativeDeviceOrientation.portraitUp => 0,
      NativeDeviceOrientation.portraitDown => 2,
      NativeDeviceOrientation.landscapeLeft => -1,
      NativeDeviceOrientation.landscapeRight => 1,
      NativeDeviceOrientation.unknown => 0,
    };
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
        color: context.theme.appBarTheme.backgroundColor?.withOpacity(0.9),
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
    final bool isOn = cameraController.value.torchState.isOn;
    final buttonRadius = BorderRadius.circular(200);
    return Positioned(
      bottom: 32,
      left: 0,
      right: 0,
      child: Center(
        child: Material(
          color: Colors.white,
          borderRadius: buttonRadius,
          child: Semantics(
            button: true,
            child: InkWell(
              borderRadius: buttonRadius,
              onTap: () => _toggleFlashLight(context),
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 16),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Semantics(
                      attributedLabel:
                          (isOn ? context.l10n.generalOn : context.l10n.generalOff).toAttributedString(context),
                      excludeSemantics: true,
                      child: Icon(
                        isOn ? Icons.flashlight_on_outlined : Icons.flashlight_off_outlined,
                        color: context.colorScheme.onSecondary,
                        size: 16,
                      ),
                    ),
                    const SizedBox(width: 12),
                    Text.rich(
                      (isOn ? context.l10n.qrScreenDisableTorchCta : context.l10n.qrScreenEnableTorchCta)
                          .toTextSpan(context),
                      style: context.textTheme.labelLarge?.copyWith(color: context.colorScheme.onSecondary),
                    ),
                  ],
                ),
              ),
            ),
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
