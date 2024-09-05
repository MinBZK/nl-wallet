import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/icon/help_icon_button.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/tertiary_button.dart';
import '../../common/widget/paragraphed_sliver_list.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';
import '../../common/widget/wallet_scrollbar.dart';

const _kRequestDigidUrl = 'https://www.digid.nl/aanvragen-en-activeren/digid-aanvragen';

class WalletPersonalizeNoDigidScreen extends StatelessWidget {
  const WalletPersonalizeNoDigidScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('personalizeNoDigidScreen'),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: WalletScrollbar(
                child: CustomScrollView(
                  slivers: [
                    SliverWalletAppBar(
                      title: context.l10n.walletPersonalizeNoDigidPageHeadline,
                      scrollController: PrimaryScrollController.maybeOf(context),
                      actions: const [HelpIconButton()],
                    ),
                    SliverPadding(
                      padding: const EdgeInsets.symmetric(horizontal: 16),
                      sliver: ParagraphedSliverList.splitContent(
                        context.l10n.walletPersonalizeNoDigidPageDescription,
                      ),
                    ),
                  ],
                ),
              ),
            ),
            _buildBottomSection(context),
          ],
        ),
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
          primaryButton: PrimaryButton(
            key: const Key('applyForDigidCta'),
            onPressed: _openRequestDigidUrl,
            text: Text.rich(context.l10n.walletPersonalizeNoDigidPageRequestDigidCta.toTextSpan(context)),
            icon: const Icon(Icons.arrow_forward_rounded),
          ),
          secondaryButton: TertiaryButton(
            onPressed: () => Navigator.maybePop(context),
            text: Text.rich(context.l10n.generalBottomBackCta.toTextSpan(context)),
            icon: const Icon(Icons.arrow_back_rounded),
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
