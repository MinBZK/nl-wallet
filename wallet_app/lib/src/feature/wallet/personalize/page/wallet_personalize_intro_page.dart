import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../../wallet_assets.dart';
import '../../../common/widget/button/confirm/confirm_button.dart';
import '../../../common/widget/button/confirm/confirm_buttons.dart';
import '../../../common/widget/sliver_wallet_app_bar.dart';
import '../../../common/widget/text/body_text.dart';

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
      child: Column(
        children: [
          Expanded(
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
                        BodyText(context.l10n.walletPersonalizeIntroPageDescription),
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
                ],
              ),
            ),
          ),
          _buildBottomSection(context),
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
        ConfirmButtons(
          primaryButton: ConfirmButton(
            key: const Key('loginWithDigidCta'),
            onPressed: onLoginWithDigidPressed,
            text: context.l10n.walletPersonalizeIntroPageLoginWithDigidCta,
            buttonType: ConfirmButtonType.primary,
            iconProvider: const AssetImage(WalletAssets.logo_digid),
          ),
          secondaryButton: ConfirmButton(
            key: const Key('noDigidCta'),
            onPressed: onNoDigidPressed,
            icon: Icons.help_outline_rounded,
            text: context.l10n.walletPersonalizeIntroPageNoDigidCta,
            buttonType: ConfirmButtonType.text,
          ),
        ),
      ],
    );
  }
}
