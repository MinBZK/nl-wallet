import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/text_icon_button.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../common/widget/sliver_sized_box.dart';

const _kDigidLogoPath = 'assets/images/digid_logo.png';

class WalletPersonalizeNoDigidScreen extends StatelessWidget {
  const WalletPersonalizeNoDigidScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
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
                  child: Text(
                    context.l10n.walletPersonalizeNoDigidPageHeadline,
                    textAlign: TextAlign.start,
                    style: context.textTheme.displaySmall,
                  ),
                ),
                const SliverSizedBox(height: 8),
                SliverToBoxAdapter(
                  child: Text(
                    context.l10n.walletPersonalizeNoDigidPageDescription,
                    textAlign: TextAlign.start,
                    style: context.textTheme.bodyLarge,
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
          onPressed: () => PlaceholderScreen.show(context),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Image.asset(_kDigidLogoPath),
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

  static void show(BuildContext context) {
    Navigator.push(
      context,
      MaterialPageRoute(builder: (c) => const WalletPersonalizeNoDigidScreen()),
    );
  }
}
