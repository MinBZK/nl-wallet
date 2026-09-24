import 'package:flutter/material.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../../../theme/light_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';

/// A widget that displays a QR code with the wallet logo embedded in the center.
class WalletQrView extends StatelessWidget {
  /// The data to be encoded in the QR code.
  final String data;

  /// Screen reader label: what the user should do with this code.
  final String semanticsLabel;

  const WalletQrView({
    required this.data,
    required this.semanticsLabel,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    // Replaces qr_flutter's own label, a hardcoded English 'qr code' without the image role.
    return Semantics(
      image: true,
      label: semanticsLabel,
      excludeSemantics: true,
      child: _buildQr(context),
    );
  }

  Widget _buildQr(BuildContext context) {
    return QrImageView(
      padding: const EdgeInsets.all(16),
      backgroundColor: LightWalletTheme.colorScheme.surface,
      size: context.isLandscape ? (context.mediaQuery.size.width * 0.3) : null,
      dataModuleStyle: const QrDataModuleStyle(
        color: Colors.black,
        dataModuleShape: QrDataModuleShape.square,
      ),
      data: data,
      embeddedImage: const AssetImage(WalletAssets.logo_wallet_qr),
      embeddedImageEmitsError: true,
      errorCorrectionLevel: QrErrorCorrectLevel.Q,
      embeddedImageStyle: const QrEmbeddedImageStyle(size: Size(64, 64)),
    );
  }
}
