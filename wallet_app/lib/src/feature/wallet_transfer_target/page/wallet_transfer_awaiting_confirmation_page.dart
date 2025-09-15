import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/widget/button/button_content.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/page_illustration.dart';
import '../../common/widget/text/body_text.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_scrollbar.dart';

class WalletTransferAwaitingConfirmationPage extends StatelessWidget {
  final VoidCallback onCtaPressed;

  const WalletTransferAwaitingConfirmationPage({
    required this.onCtaPressed,
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
                        TitleText(context.l10n.walletTransferAwaitingConfirmationPageTitle),
                        const SizedBox(height: 8),
                        BodyText(context.l10n.walletTransferAwaitingConfirmationPageDescription),
                        const SizedBox(height: 8),
                      ],
                    ),
                  ),
                  const PageIllustration(asset: WalletAssets.svg_move_destination_permission),
                ],
              ),
            ),
            ListButton(
              onPressed: onCtaPressed,
              icon: const Icon(Icons.close_outlined),
              mainAxisAlignment: MainAxisAlignment.center,
              iconPosition: IconPosition.start,
              dividerSide: DividerSide.top,
              text: Text.rich(context.l10n.walletTransferAwaitingConfirmationPageCta.toTextSpan(context)),
            ),
          ],
        ),
      ),
    );
  }
}
