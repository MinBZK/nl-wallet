import 'package:flutter/material.dart';

import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../screen/placeholder_screen.dart';
import '../widget/button/bottom_close_button.dart';
import '../widget/menu_item.dart';
import '../widget/text/title_text.dart';
import '../widget/wallet_scrollbar.dart';

class QrActionSheet extends StatelessWidget {
  const QrActionSheet({super.key});

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
            child: TitleText(context.l10n.qrActionSheetTitle),
          ),
          const Divider(),
          MenuItem(
            leftIcon: Image.asset(WalletAssets.icon_qr_scan, color: context.theme.iconTheme.color),
            label: Text(context.l10n.qrActionSheetScanQrTitle),
            subtitle: Text(context.l10n.qrActionSheetScanQrDescription),
            onPressed: () {
              Navigator.pop(context);
              Navigator.pushNamed(context, WalletRoutes.qrScanRoute);
            },
          ),
          const Divider(),
          MenuItem(
            leftIcon: const Icon(Icons.qr_code),
            label: Text(context.l10n.qrActionSheetShowQrTitle),
            subtitle: Text(context.l10n.qrActionSheetShowQrDescription),
            onPressed: () {
              Navigator.pop(context);
              PlaceholderScreen.showGeneric(context);
            },
          ),
          const BottomCloseButton(),
        ],
      ),
    );
  }

  static Future<void> show(BuildContext context) {
    return showModalBottomSheet<void>(
      context: context,
      isDismissible: !context.isScreenReaderEnabled,
      isScrollControlled: true,
      builder: (BuildContext context) {
        return const WalletScrollbar(
          child: SingleChildScrollView(
            child: QrActionSheet(),
          ),
        );
      },
    );
  }
}
