import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../common/widget/button/text_icon_button.dart';
import '../../../common/widget/sliver_sized_box.dart';

const _kIllustrationPath = 'assets/images/personalize_wallet_intro_illustration.png';
const _kDigidLogoPath = 'assets/images/digid_logo.png';

class WalletPersonalizeIntroPage extends StatelessWidget {
  final VoidCallback onLoginWithDigidPressed;
  final VoidCallback onNoDigidPressed;

  const WalletPersonalizeIntroPage({
    required this.onLoginWithDigidPressed,
    required this.onNoDigidPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    final theme = Theme.of(context);
    return Scrollbar(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16),
        child: CustomScrollView(
          slivers: [
            const SliverSizedBox(height: 36),
            SliverToBoxAdapter(
              child: Text(
                locale.walletPersonalizeIntroPageTitle,
                textAlign: TextAlign.start,
                style: theme.textTheme.displaySmall,
              ),
            ),
            const SliverSizedBox(height: 8),
            SliverToBoxAdapter(
              child: Text(
                locale.walletPersonalizeIntroPageDescription,
                textAlign: TextAlign.start,
                style: theme.textTheme.bodyLarge,
              ),
            ),
            const SliverSizedBox(height: 32),
            SliverToBoxAdapter(
              child: SizedBox(
                width: double.infinity,
                child: Image.asset(
                  _kIllustrationPath,
                  fit: BoxFit.fitWidth,
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
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Column(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        ElevatedButton(
          onPressed: onLoginWithDigidPressed,
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Image.asset(_kDigidLogoPath),
              const SizedBox(width: 12),
              Flexible(
                child: Text(locale.walletPersonalizeIntroPageLoginWithDigidCta),
              ),
            ],
          ),
        ),
        const SizedBox(height: 8),
        Center(
          child: TextIconButton(
            onPressed: onNoDigidPressed,
            child: Text(locale.walletPersonalizeIntroPageNoDigidCta),
          ),
        ),
        const SizedBox(height: 24),
      ],
    );
  }
}
