import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../../wallet_assets.dart';
import '../../../common/widget/button/primary_button.dart';
import '../../../common/widget/button/secondary_button.dart';
import '../../../common/widget/sliver_wallet_app_bar.dart';

class WalletPersonalizeIntroPage extends StatelessWidget {
  final VoidCallback onLoginWithDigidPressed;
  final VoidCallback onNoDigidPressed;
  final double? progress;

  const WalletPersonalizeIntroPage({
    required this.onLoginWithDigidPressed,
    required this.onNoDigidPressed,
    this.progress,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      child: CustomScrollView(
        slivers: [
          SliverWalletAppBar(
            title: context.l10n.walletPersonalizeIntroPageTitle,
            progress: progress,
          ),
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            sliver: SliverList(
                delegate: SliverChildListDelegate([
              Text(
                context.l10n.walletPersonalizeIntroPageDescription,
                textAlign: TextAlign.start,
                style: context.textTheme.bodyLarge,
              ),
              const SizedBox(height: 32),
              SizedBox(
                width: double.infinity,
                child: Image.asset(
                  WalletAssets.illustration_personalize_wallet_intro,
                  fit: BoxFit.fitWidth,
                ),
              ),
              const SizedBox(height: 32),
            ])),
          ),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: _buildBottomSection(context),
          ),
        ],
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
                key: const Key('loginWithDigidCta'),
                onPressed: onLoginWithDigidPressed,
                text: context.l10n.walletPersonalizeIntroPageLoginWithDigidCta,
              ),
              const SizedBox(height: 12),
              SecondaryButton(
                key: const Key('noDigidCta'),
                onPressed: onNoDigidPressed,
                text: context.l10n.walletPersonalizeIntroPageNoDigidCta,
              ),
            ],
          ),
        ),
      ],
    );
  }
}
