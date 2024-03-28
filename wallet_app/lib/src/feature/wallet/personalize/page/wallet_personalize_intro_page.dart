import 'dart:math';

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
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      bottom: false, //handled in _buildBottomSection,
      child: Scrollbar(
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
                ]),
              ),
            ),
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
      mainAxisSize: MainAxisSize.min,
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        const Divider(height: 1),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Column(
            children: [
              SizedBox(height: context.isLandscape ? 8 : 24),
              PrimaryButton(
                key: const Key('loginWithDigidCta'),
                onPressed: onLoginWithDigidPressed,
                text: context.l10n.walletPersonalizeIntroPageLoginWithDigidCta,
              ),
              const SizedBox(height: 12),
              SecondaryButton(
                key: const Key('noDigidCta'),
                onPressed: onNoDigidPressed,
                icon: Icons.help_outline_rounded,
                text: context.l10n.walletPersonalizeIntroPageNoDigidCta,
              ),
              SizedBox(height: max(24, context.mediaQuery.viewPadding.bottom))
            ],
          ),
        ),
      ],
    );
  }
}
