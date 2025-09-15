import 'dart:math';

import 'package:flutter/material.dart';
import 'package:qr_flutter/qr_flutter.dart';

import '../../../theme/light_wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/widget/button/button_content.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/text/title_text.dart';

const _kDialogRadius = Radius.circular(8);
const _kDialogBorderRadius = BorderRadius.only(
  topLeft: _kDialogRadius,
  topRight: _kDialogRadius,
  bottomRight: _kDialogRadius,
  bottomLeft: _kDialogRadius,
);

class QrCodeDialog extends StatelessWidget {
  final String data;

  const QrCodeDialog({required this.data, super.key});

  @override
  Widget build(BuildContext context) {
    final scaleFactor = context.isLandscape ? 0.6 : 0.8;
    final maxQrSize = min(context.mediaQuery.size.width, context.mediaQuery.size.height) * scaleFactor;
    return SimpleDialog(
      contentPadding: EdgeInsets.zero,
      insetPadding: const EdgeInsets.all(16),
      shape: const RoundedRectangleBorder(borderRadius: _kDialogBorderRadius),
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
          child: Column(
            children: [
              TitleText(
                context.l10n.qrCodeCodeDialogTitle,
                textAlign: TextAlign.center,
                style: context.textTheme.headlineSmall,
              ),
              const SizedBox(height: 24),
              Container(
                alignment: Alignment.center,
                width: maxQrSize,
                height: maxQrSize,
                child: QrImageView(
                  padding: const EdgeInsets.all(12),
                  backgroundColor: LightWalletTheme.colorScheme.surface,
                  dataModuleStyle: const QrDataModuleStyle(
                    color: Colors.black,
                    dataModuleShape: QrDataModuleShape.square,
                  ),
                  data: data,
                  embeddedImage: const AssetImage(WalletAssets.logo_wallet),
                  embeddedImageEmitsError: true,
                  embeddedImageStyle: const QrEmbeddedImageStyle(size: Size(32, 32)),
                ),
              ),
            ],
          ),
        ),
        ClipRRect(
          borderRadius: const BorderRadiusGeometry.only(bottomLeft: _kDialogRadius, bottomRight: _kDialogRadius),
          child: ListButton(
            onPressed: () => Navigator.pop(context),
            icon: const Icon(Icons.close_outlined),
            mainAxisAlignment: MainAxisAlignment.center,
            iconPosition: IconPosition.start,
            dividerSide: DividerSide.top,
            text: Text.rich(context.l10n.generalClose.toTextSpan(context)),
          ),
        ),
      ],
    );
  }

  static Future<void> show(BuildContext context, {required String data}) {
    return showDialog<void>(
      context: context,
      builder: (BuildContext context) => QrCodeDialog(data: data),
    );
  }
}
