import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:url_launcher/url_launcher_string.dart';

import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../util/launch_util.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/list_button.dart';
import '../common/widget/menu_item.dart';
import '../common/widget/mock_indicator_text.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text_with_link.dart';
import '../common/widget/version/config_version_text.dart';
import '../common/widget/version/string_version_text.dart';
import '../common/widget/wallet_scrollbar.dart';

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
                    SliverWalletAppBar(
                      title: context.l10n.aboutScreenTitle,
                      scrollController: PrimaryScrollController.maybeOf(context),
                    ),
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
        MenuItem(
          label: Text.rich(context.l10n.aboutScreenPrivacyCta.toTextSpan(context)),
          onPressed: () => Navigator.pushNamed(context, WalletRoutes.privacyPolicyRoute),
          dividerSide: DividerSide.both,
        ),
        const Padding(
          padding: EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              StringVersionText(),
              SizedBox(height: 8),
              ConfigVersionText(),
              SizedBox(height: 8),
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
    final paragraphs = fullText.split('\n\n');
    return ListView.separated(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemBuilder: (c, i) {
        final paragraph = paragraphs[i];
        if (paragraph.contains(url)) {
          return TextWithLink(
            fullText: paragraph,
            linkText: url,
            onTapHint: context.l10n.generalWCAGOpenLink,
            onLinkPressed: () {
              final urlString = url.startsWith('http') ? url : 'https://$url';
              launchUrlStringCatching(urlString, mode: LaunchMode.externalApplication);
            },
          );
        } else {
          return BodyText(paragraphs[i]);
        }
      },
      separatorBuilder: (BuildContext context, int index) => const SizedBox(height: 8),
      itemCount: paragraphs.length,
    );
  }
}
