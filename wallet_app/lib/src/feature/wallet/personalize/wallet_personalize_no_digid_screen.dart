import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/secondary_button.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';

const _kRequestDigidUrl = 'https://www.digid.nl/aanvragen-en-activeren/digid-aanvragen';

class WalletPersonalizeNoDigidScreen extends StatelessWidget {
  const WalletPersonalizeNoDigidScreen({super.key});

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
    return Column(
      mainAxisSize: MainAxisSize.min,
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        const Divider(height: 1),
        Padding(
          padding: EdgeInsets.symmetric(horizontal: 16, vertical: context.isLandscape ? 8 : 24),
          child: Column(
            children: [
              PrimaryButton(
                key: const Key('applyForDigidCta'),
                onPressed: () => _openRequestDigidUrl(),
                text: context.l10n.walletPersonalizeNoDigidPageRequestDigidCta,
              ),
              const SizedBox(height: 12),
              SecondaryButton(
                onPressed: () => Navigator.maybePop(context),
                text: context.l10n.generalBottomBackCta,
                icon: Icons.arrow_back,
              ),
            ],
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
