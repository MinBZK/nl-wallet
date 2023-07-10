import 'package:flutter/material.dart';

import '../../../../util/extension/build_context_extension.dart';
import '../../../common/widget/button/text_icon_button.dart';
import '../../../common/widget/sliver_sized_box.dart';

const _kDigidLogoPath = 'assets/images/digid_logo.png';

class WalletPersonalizeDigidErrorPage extends StatelessWidget {
  final VoidCallback onRetryPressed;
  final VoidCallback onHelpPressed;

  const WalletPersonalizeDigidErrorPage({
    required this.onRetryPressed,
    required this.onHelpPressed,
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
            SliverSizedBox(
              height: 105,
              child: ExcludeSemantics(
                child: Container(
                  decoration: BoxDecoration(
                    color: context.colorScheme.secondaryContainer,
                    borderRadius: BorderRadius.circular(8),
                  ),
                  alignment: Alignment.center,
                  child: const Text('Placeholder image'),
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
                      context.l10n.walletPersonalizeDigidErrorPageTitle,
                      textAlign: TextAlign.start,
                      style: context.textTheme.displaySmall,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      context.l10n.walletPersonalizeDigidErrorPageDescription,
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
                _kDigidLogoPath,
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
        const SizedBox(height: 24),
      ],
    );
  }
}
