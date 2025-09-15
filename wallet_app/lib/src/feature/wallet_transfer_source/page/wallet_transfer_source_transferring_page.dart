import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/widget/animation/wallet_lottie_player.dart';
import '../../common/widget/button/button_content.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/text/body_text.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_scrollbar.dart';

class WalletTransferSourceTransferringPage extends StatelessWidget {
  final VoidCallback onStopPressed;

  const WalletTransferSourceTransferringPage({required this.onStopPressed, super.key});

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: WalletScrollbar(
        child: LayoutBuilder(
          builder: (context, constraints) {
            return SingleChildScrollView(
              child: ConstrainedBox(
                constraints: BoxConstraints(minHeight: constraints.maxHeight),
                child: IntrinsicHeight(
                  child: Column(
                    children: [
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
                        child: Column(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            TitleText(context.l10n.walletTransferScreenTransferringTitle),
                            const SizedBox(height: 8),
                            BodyText(context.l10n.walletTransferScreenTransferringDescription),
                            const SizedBox(height: 8),
                          ],
                        ),
                      ),
                      Expanded(
                        child: Container(
                          width: double.infinity,
                          decoration: BoxDecoration(
                            color: context.colorScheme.primaryContainer,
                          ),
                          child: const WalletLottiePlayer(asset: WalletAssets.lottie_generic_loader),
                        ),
                      ),
                      ListButton(
                        onPressed: onStopPressed,
                        icon: const Icon(Icons.close_outlined),
                        mainAxisAlignment: MainAxisAlignment.center,
                        iconPosition: IconPosition.start,
                        dividerSide: DividerSide.top,
                        text: Text.rich(context.l10n.generalStop.toTextSpan(context)),
                      ),
                    ],
                  ),
                ),
              ),
            );
          },
        ),
      ),
    );
  }
}
