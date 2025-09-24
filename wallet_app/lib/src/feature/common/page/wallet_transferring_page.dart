import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_assets.dart';
import '../widget/animation/wallet_lottie_player.dart';
import '../widget/button/button_content.dart';
import '../widget/button/list_button.dart';
import '../widget/text/body_text.dart';
import '../widget/text/title_text.dart';
import '../widget/wallet_scrollbar.dart';

class WalletTransferringPage extends StatelessWidget {
  final String title;
  final String description;
  final String cta;
  final VoidCallback onCtaPressed;

  const WalletTransferringPage({
    required this.title,
    required this.description,
    required this.cta,
    required this.onCtaPressed,
    super.key,
  });

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
                            TitleText(title),
                            const SizedBox(height: 8),
                            BodyText(description),
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
                        onPressed: onCtaPressed,
                        icon: const Icon(Icons.close_outlined),
                        mainAxisAlignment: MainAxisAlignment.center,
                        iconPosition: IconPosition.start,
                        dividerSide: DividerSide.top,
                        text: Text.rich(cta.toTextSpan(context)),
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
