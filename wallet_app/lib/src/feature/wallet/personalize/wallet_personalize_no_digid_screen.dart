import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/confirm/confirm_button.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/icon/help_icon_button.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';
import '../../common/widget/text/body_text.dart';

const _kRequestDigidUrl = 'https://www.digid.nl/aanvragen-en-activeren/digid-aanvragen';

class WalletPersonalizeNoDigidScreen extends StatelessWidget {
  const WalletPersonalizeNoDigidScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('personalizeNoDigidScreen'),
      body: Column(
        children: [
          Expanded(
            child: Scrollbar(
              child: CustomScrollView(
                slivers: [
                  SliverWalletAppBar(
                    title: context.l10n.walletPersonalizeNoDigidPageHeadline,
                    actions: const [HelpIconButton()],
                  ),
                  SliverSafeArea(
                    top: false,
                    bottom: false,
                    sliver: SliverPadding(
                      padding: const EdgeInsets.symmetric(horizontal: 16),
                      sliver: SliverToBoxAdapter(
                        child: BodyText(context.l10n.walletPersonalizeNoDigidPageDescription),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
          _buildBottomSection(context),
        ],
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        const Divider(height: 1),
        ConfirmButtons(
          forceVertical: !context.isLandscape,
          primaryButton: ConfirmButton(
            key: const Key('applyForDigidCta'),
            onPressed: () => _openRequestDigidUrl(),
            text: context.l10n.walletPersonalizeNoDigidPageRequestDigidCta,
            buttonType: ConfirmButtonType.primary,
            icon: Icons.arrow_forward_rounded,
          ),
          secondaryButton: ConfirmButton(
            onPressed: () => Navigator.maybePop(context),
            text: context.l10n.generalBottomBackCta,
            icon: Icons.arrow_back_rounded,
            buttonType: ConfirmButtonType.text,
          ),
        ),
      ],
    );
  }

  void _openRequestDigidUrl() => launchUrlString(_kRequestDigidUrl, mode: LaunchMode.externalApplication);

  static void show(BuildContext context) {
    Navigator.push(
      context,
      MaterialPageRoute(builder: (c) => const WalletPersonalizeNoDigidScreen()),
    );
  }
}
