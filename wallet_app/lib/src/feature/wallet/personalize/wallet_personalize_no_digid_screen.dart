import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/button/text_icon_button.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../common/widget/sliver_sized_box.dart';

const _kDigidLogoPath = 'assets/images/digid_logo.png';

class WalletPersonalizeNoDigidScreen extends StatelessWidget {
  const WalletPersonalizeNoDigidScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    final theme = Theme.of(context);
    return Scaffold(
      appBar: AppBar(
        title: Text(locale.walletPersonalizeNoDigidPageTitle),
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
                    locale.walletPersonalizeNoDigidPageHeadline,
                    textAlign: TextAlign.start,
                    style: theme.textTheme.displaySmall,
                  ),
                ),
                const SliverSizedBox(height: 8),
                SliverToBoxAdapter(
                  child: Text(
                    locale.walletPersonalizeNoDigidPageDescription,
                    textAlign: TextAlign.start,
                    style: theme.textTheme.bodyLarge,
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
    final locale = AppLocalizations.of(context);
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
                child: Text(locale.walletPersonalizeNoDigidPageRequestDigidCta),
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
            child: Text(locale.walletPersonalizeNoDigidPageBackCta),
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
