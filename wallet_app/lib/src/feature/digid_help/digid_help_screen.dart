import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../navigation/secured_page_route.dart';
import '../../util/extension/build_context_extension.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/link_button.dart';
import '../common/widget/sliver_divider.dart';
import '../common/widget/sliver_sized_box.dart';

final kRequestDigidUri = Uri.parse('https://www.digid.nl/aanvragen-en-activeren/digid-aanvragen');
final kDigidHelpUri = Uri.parse('https://www.digid.nl/hulp/');

class DigidHelpScreen extends StatelessWidget {
  final String title;

  const DigidHelpScreen({required this.title, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(title),
      ),
      body: SafeArea(
        child: _buildBody(context),
      ),
    );
  }

  static void show(BuildContext context, {required String title, bool secured = true}) {
    Navigator.push(
      context,
      secured
          ? SecuredPageRoute(builder: (c) => DigidHelpScreen(title: title))
          : MaterialPageRoute(builder: (c) => DigidHelpScreen(title: title)),
    );
  }

  Widget _buildBody(BuildContext context) {
    return CustomScrollView(
      slivers: [
        const SliverSizedBox(height: 8),
        SliverPadding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
          sliver: SliverToBoxAdapter(
            child: Text(
              context.l10n.digidHelpScreenTitle,
              style: context.textTheme.displayMedium,
            ),
          ),
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
        const SliverDivider(),
        const SliverFillRemaining(
          fillOverscroll: true,
          hasScrollBody: false,
          child: BottomBackButton(showDivider: true),
        ),
        const SliverSizedBox(height: 32),
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
          onPressed: () => launchUrl(kRequestDigidUri, mode: LaunchMode.externalApplication),
          customPadding: EdgeInsets.zero,
          child: Text(context.l10n.digidHelpScreenNoDigidCta),
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
          onPressed: () => launchUrl(kDigidHelpUri, mode: LaunchMode.externalApplication),
          customPadding: EdgeInsets.zero,
          child: Text(context.l10n.digidHelpScreenHelpNeededCta),
        ),
      ],
    );
  }
}
