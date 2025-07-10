import 'package:flutter/material.dart';

import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_assets.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/button/confirm/confirm_buttons.dart';
import '../common/widget/button/list_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/button/secondary_button.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/spacer/sliver_sized_box.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_scrollbar.dart';

class FraudCheckPage extends StatelessWidget {
  /// Callback that is triggered when the user approves the request
  final VoidCallback onAcceptPressed;

  /// Callback that is triggered when the user declines the request
  final VoidCallback onDeclinePressed;

  /// The url from which the user should have opened the flow. Prominently displayed for the user to check
  final String originUrl;

  const FraudCheckPage({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    required this.originUrl,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: WalletScrollbar(
        child: CustomScrollView(
          slivers: <Widget>[
            const SliverSizedBox(height: 24),
            SliverToBoxAdapter(child: _buildTextSection(context)),
            const SliverSizedBox(height: 24),
            const SliverToBoxAdapter(child: PageIllustration(asset: WalletAssets.svg_url_check)),
            const SliverSizedBox(height: 32),
            SliverToBoxAdapter(
              child: ListButton(
                onPressed: () => PlaceholderScreen.showGeneric(context),
                text: Text.rich(context.l10n.fraudCheckPageAboutCta.toTextSpan(context)),
                icon: const Icon(Icons.help_outline_outlined),
              ),
            ),
            const SliverSizedBox(height: 32),
            SliverFillRemaining(
              hasScrollBody: false,
              fillOverscroll: true,
              child: Column(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  const Divider(),
                  ConfirmButtons(
                    primaryButton: PrimaryButton(
                      onPressed: onAcceptPressed,
                      text: Text.rich(context.l10n.fraudCheckPageContinueCta.toTextSpan(context)),
                      icon: const Icon(Icons.check_outlined),
                    ),
                    secondaryButton: SecondaryButton(
                      onPressed: onDeclinePressed,
                      text: Text.rich(context.l10n.fraudCheckPageStopCta.toTextSpan(context)),
                      icon: const Icon(Icons.block_flipped),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildTextSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          TitleText(context.l10n.fraudCheckPageTitle(originUrl)),
          const SizedBox(height: 8),
          BodyText(context.l10n.fraudCheckPageDescription),
        ],
      ),
    );
  }
}
