import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/widget/button/button_content.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/page_illustration.dart';
import '../../common/widget/paragraphed_list.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_scrollbar.dart';

class WalletTransferSourceTransferSuccessPage extends StatelessWidget {
  final VoidCallback onCtaPressed;

  const WalletTransferSourceTransferSuccessPage({required this.onCtaPressed, super.key});

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildScrollableSection(context),
          _buildBottomSection(context),
        ],
      ),
    );
  }

  Widget _buildScrollableSection(BuildContext context) {
    return Expanded(
      child: WalletScrollbar(
        child: ListView(
          padding: const EdgeInsets.symmetric(vertical: 12),
          children: [
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  TitleText(context.l10n.walletTransferSourceScreenSuccessTitle),
                  const SizedBox(height: 8),
                  ParagraphedList.splitContent(
                    context.l10n.walletTransferSourceScreenSuccessDescription,
                  ),
                ],
              ),
            ),
            const SizedBox(height: 24),
            const PageIllustration(asset: WalletAssets.svg_move_source_success),
          ],
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return ListButton(
      dividerSide: DividerSide.top,
      icon: const Icon(Icons.arrow_forward_outlined),
      iconPosition: IconPosition.start,
      mainAxisAlignment: MainAxisAlignment.center,
      onPressed: onCtaPressed,
      text: Text.rich(context.l10n.walletTransferSourceScreenSuccessCta.toTextSpan(context)),
    );
  }
}
