import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
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
    return Scrollbar(
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
                      context.l10n.walletPersonalizeIntroPageTitle,
                      textAlign: TextAlign.start,
                      style: context.textTheme.displaySmall,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      context.l10n.walletPersonalizeIntroPageDescription,
                      textAlign: TextAlign.start,
                      style: context.textTheme.bodyLarge,
                    )
                  ],
                ),
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
                child: Text(context.l10n.walletPersonalizeIntroPageLoginWithDigidCta),
              ),
            ],
          ),
        ),
        const SizedBox(height: 8),
        Center(
          child: TextIconButton(
            onPressed: onNoDigidPressed,
            child: Text(context.l10n.walletPersonalizeIntroPageNoDigidCta),
          ),
        ),
        const SizedBox(height: 24),
      ],
    );
  }
}
