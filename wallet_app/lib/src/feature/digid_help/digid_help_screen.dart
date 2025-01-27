import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../navigation/secured_page_route.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../util/launch_util.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/link_button.dart';
import '../common/widget/sliver_divider.dart';
import '../common/widget/sliver_wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';

final kRequestDigidUri = Uri.parse('https://www.digid.nl/aanvragen-en-activeren/digid-aanvragen');
final kDigidHelpUri = Uri.parse('https://www.digid.nl/hulp/');

class DigidHelpScreen extends StatelessWidget {
  const DigidHelpScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: _buildBody(context),
      ),
    );
  }

  static void show(BuildContext context, {bool secured = true}) {
    Navigator.push(
      context,
      secured
          ? SecuredPageRoute(builder: (c) => const DigidHelpScreen())
          : MaterialPageRoute(builder: (c) => const DigidHelpScreen()),
    );
  }

  Widget _buildBody(BuildContext context) {
    return Column(
      children: [
        Expanded(
          child: WalletScrollbar(
            child: CustomScrollView(
              slivers: [
                SliverWalletAppBar(
                  title: context.l10n.digidHelpScreenTitle,
                  scrollController: PrimaryScrollController.maybeOf(context),
                ),
                const SliverDivider(),
                SliverPadding(
                  padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                  sliver: SliverToBoxAdapter(child: _buildNoDigidSection(context)),
                ),
                const SliverDivider(),
                SliverPadding(
                  padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                  sliver: SliverToBoxAdapter(child: _buildHelpNeededSection(context)),
                ),
              ],
            ),
          ),
        ),
        const BottomBackButton(),
      ],
    );
  }

  Widget _buildNoDigidSection(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const SizedBox(height: 12),
        Text(
          context.l10n.digidHelpScreenNoDigidTitle,
          style: context.textTheme.titleMedium,
        ),
        Text(
          context.l10n.digidHelpScreenNoDigidDescription,
          style: context.textTheme.bodyLarge,
        ),
        LinkButton(
          onPressed: () => launchUriCatching(kRequestDigidUri, mode: LaunchMode.externalApplication),
          text: Text.rich(context.l10n.digidHelpScreenNoDigidCta.toTextSpan(context)),
        ),
      ],
    );
  }

  Widget _buildHelpNeededSection(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const SizedBox(height: 12),
        Text(
          context.l10n.digidHelpScreenHelpNeededTitle,
          style: context.textTheme.titleMedium,
        ),
        Text(
          context.l10n.digidHelpScreenHelpNeededDescription,
          style: context.textTheme.bodyLarge,
        ),
        LinkButton(
          onPressed: () => launchUriCatching(kDigidHelpUri, mode: LaunchMode.externalApplication),
          text: Text.rich(context.l10n.digidHelpScreenHelpNeededCta.toTextSpan(context)),
        ),
      ],
    );
  }
}
