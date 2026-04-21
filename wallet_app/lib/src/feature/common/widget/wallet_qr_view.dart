import 'package:flutter/material.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../../../theme/light_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';

/// A widget that displays a QR code with the wallet logo embedded in the center.
class WalletQrView extends StatelessWidget {
  /// The data to be encoded in the QR code.
  final String data;

  const WalletQrView({
    required this.data,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return QrImageView(
      padding: EdgeInsets.zero,
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
