import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/widget/button/text_icon_button.dart';
import '../../common/widget/sliver_sized_box.dart';

const _kRequestDigidUrl = 'https://www.digid.nl/aanvragen-en-activeren/digid-aanvragen';

class WalletPersonalizeNoDigidScreen extends StatelessWidget {
  const WalletPersonalizeNoDigidScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('personalizeNoDigidScreen'),
      appBar: AppBar(
        title: Text(context.l10n.walletPersonalizeNoDigidPageTitle),
      ),
      body: SafeArea(
        child: Scrollbar(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: CustomScrollView(
              slivers: [
                const SliverSizedBox(height: 36),
                SliverToBoxAdapter(
                  child: MergeSemantics(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          context.l10n.walletPersonalizeNoDigidPageHeadline,
                          textAlign: TextAlign.start,
                          style: context.textTheme.displaySmall,
                        ),
                        const SizedBox(height: 8),
                        Text(
                          context.l10n.walletPersonalizeNoDigidPageDescription,
                          textAlign: TextAlign.start,
                          style: context.textTheme.bodyLarge,
                        ),
                      ],
                    ),
                  ),
                ),
                const SliverSizedBox(height: 32),
                SliverFillRemaining(
                  hasScrollBody: false,
                  fillOverscroll: true,
                  child: _buildBottomSection(context),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
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
