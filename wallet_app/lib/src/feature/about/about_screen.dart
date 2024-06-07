import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../util/extension/build_context_extension.dart';
import '../../util/launch_util.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/config_version_text.dart';
import '../common/widget/mock_indicator_text.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/text_with_link.dart';
import '../common/widget/version_text.dart';
import '../common/widget/wallet_scrollbar.dart';
import '../menu/widget/menu_row.dart';

class AboutScreen extends StatelessWidget {
  const AboutScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('aboutScreen'),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: WalletScrollbar(
                child: CustomScrollView(
                  slivers: [
                    SliverWalletAppBar(title: context.l10n.aboutScreenTitle),
                    _buildContentSliver(context),
                  ],
                ),
              ),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildContentSliver(BuildContext context) {
    return SliverList.list(
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: _buildDescription(context),
        ),
        const SizedBox(height: 16),
        const Divider(height: 1),
        MenuRow(
          label: context.l10n.aboutScreenPrivacyCta,
          onTap: () => PlaceholderScreen.showGeneric(context, secured: false),
        ),
        const Divider(height: 1),
        MenuRow(
          label: context.l10n.aboutScreenTermsCta,
          onTap: () => PlaceholderScreen.showContract(context, secured: false),
        ),
        const Divider(height: 1),
        const Padding(
          padding: EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              VersionText(),
              ConfigVersionText(),
              MockIndicatorText(),
            ],
          ),
        ),
        const SizedBox(height: 24),
      ],
    );
  }

  Widget _buildDescription(BuildContext context) {
    final url = context.l10n.aboutScreenDescriptionLink;
    final fullText = context.l10n.aboutScreenDescription(url);
    return TextWithLink(
      fullText: fullText,
      ctaText: url,
      onTapHint: context.l10n.generalWCAGOpenLink,
      onCtaPressed: () {
        final urlString = url.startsWith('http') ? url : 'https://$url';
        launchUrlStringCatching(urlString, mode: LaunchMode.externalApplication);
      },
    );
  }
}
