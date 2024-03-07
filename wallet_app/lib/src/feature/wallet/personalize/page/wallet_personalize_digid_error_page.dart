import 'dart:math';

import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../../wallet_assets.dart';
import '../../../common/widget/button/text_icon_button.dart';
import '../../../common/widget/sliver_sized_box.dart';

class WalletPersonalizeDigidErrorPage extends StatelessWidget {
  final VoidCallback onRetryPressed;
  final VoidCallback onHelpPressed;
  final String title, description;

  const WalletPersonalizeDigidErrorPage({
    required this.onRetryPressed,
    required this.onHelpPressed,
    required this.title,
    required this.description,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      bottom: false, //handled by _buildBottomSection
      child: Scrollbar(
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: CustomScrollView(
            slivers: [
              const SliverSizedBox(height: 36),
              SliverToBoxAdapter(
                child: ExcludeSemantics(
                  child: Image.asset(
                    WalletAssets.illustration_digid_failure,
                    fit: context.isLandscape ? BoxFit.contain : BoxFit.fitWidth,
                    height: context.isLandscape ? 160 : null,
                    width: double.infinity,
                  ),
                ),
              ),
              const SliverSizedBox(height: 24),
              SliverToBoxAdapter(
                child: MergeSemantics(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        title,
                        textAlign: TextAlign.start,
                        style: context.textTheme.displaySmall,
                      ),
                      const SizedBox(height: 8),
                      Text(
                        description,
                        textAlign: TextAlign.start,
                        style: context.textTheme.bodyLarge,
                      ),
                    ],
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
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Column(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        ElevatedButton(
          onPressed: onRetryPressed,
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Image.asset(
                WalletAssets.logo_digid,
                excludeFromSemantics: true,
              ),
              const SizedBox(width: 12),
              Flexible(
                child: Text(context.l10n.walletPersonalizeDigidErrorPageLoginWithDigidCta),
              ),
            ],
          ),
        ),
        const SizedBox(height: 8),
        Center(
          child: TextIconButton(
            onPressed: onHelpPressed,
            child: Text(
              context.l10n.walletPersonalizeDigidErrorPageNoDigidCta,
            ),
          ),
        ),
        SizedBox(height: max(24, context.mediaQuery.viewPadding.bottom)),
      ],
    );
  }
}
