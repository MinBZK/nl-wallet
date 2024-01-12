import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/button/text_icon_button.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';

const _kRequestDigidUrl = 'https://www.digid.nl/aanvragen-en-activeren/digid-aanvragen';

class WalletPersonalizeNoDigidScreen extends StatelessWidget {
  const WalletPersonalizeNoDigidScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('personalizeNoDigidScreen'),
      body: Scrollbar(
        child: CustomScrollView(
          slivers: [
            SliverWalletAppBar(
              title: context.l10n.walletPersonalizeNoDigidPageHeadline,
              actions: [
                IconButton(
                  onPressed: () => PlaceholderScreen.show(context),
                  icon: const Icon(Icons.help_outline_rounded),
                )
              ],
            ),
            SliverPadding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              sliver: SliverToBoxAdapter(
                child: Text(
                  context.l10n.walletPersonalizeNoDigidPageDescription,
                  textAlign: TextAlign.start,
                  style: context.textTheme.bodyLarge,
                ),
              ),
            ),
            SliverFillRemaining(
              hasScrollBody: false,
              fillOverscroll: true,
              child: _buildBottomSection(context),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          ElevatedButton(
            key: const Key('applyForDigidCta'),
            onPressed: () => _openRequestDigidUrl(),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Image.asset(WalletAssets.logo_digid),
                const SizedBox(width: 12),
                Flexible(
                  child: Text(context.l10n.walletPersonalizeNoDigidPageRequestDigidCta),
                ),
              ],
            ),
          ),
          const SizedBox(height: 8),
          Center(
            child: TextIconButton(
              icon: Icons.arrow_back,
              iconPosition: IconPosition.start,
              onPressed: () => Navigator.pop(context),
              child: Text(context.l10n.walletPersonalizeNoDigidPageBackCta),
            ),
          ),
          const SizedBox(height: 24),
        ],
      ),
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
