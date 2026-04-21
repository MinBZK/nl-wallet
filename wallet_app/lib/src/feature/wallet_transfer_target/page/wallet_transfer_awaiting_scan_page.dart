import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/dialog/qr_code_dialog.dart';
import '../../common/widget/button/button_content.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/text/body_text.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_qr_view.dart';
import '../../common/widget/wallet_scrollbar.dart';

class WalletTransferAwaitingScanPage extends StatelessWidget {
  final VoidCallback onBackPressed;
  final String data;

  const WalletTransferAwaitingScanPage({
    required this.onBackPressed,
    required this.data,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: WalletScrollbar(
        child: Column(
          children: [
            Expanded(
              child: ListView(
                children: [
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        TitleText(context.l10n.walletTransferAwaitingScanPageTitle),
                        const SizedBox(height: 8),
                        BodyText(context.l10n.walletTransferAwaitingScanPageDescription),
                        const SizedBox(height: 8),
                      ],
                    ),
                  ),
                  ListButton(
                    onPressed: () => QrCodeDialog.show(context, title: context.l10n.qrCodeCodeDialogTitle, data: data),
                    icon: const Icon(Icons.arrow_forward_ios_outlined),
                    text: Text.rich(context.l10n.walletTransferAwaitingScanPageCenterQrCta.toTextSpan(context)),
                  ),
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
                    child: Center(child: WalletQrView(data: data)),
                  ),
                ],
              ),
            ),
            ListButton(
              onPressed: onBackPressed,
              icon: const Icon(Icons.arrow_back_outlined),
              mainAxisAlignment: MainAxisAlignment.center,
              iconPosition: IconPosition.start,
              dividerSide: DividerSide.top,
              text: Text.rich(context.l10n.generalBottomBackCta.toTextSpan(context)),
            ),
          ],
        ),
      ),
    );
  }
}
