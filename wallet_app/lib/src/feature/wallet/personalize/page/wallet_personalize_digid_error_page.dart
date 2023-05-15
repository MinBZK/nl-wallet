import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

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
    final locale = AppLocalizations.of(context);
    return Scrollbar(
      thumbVisibility: true,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16),
        child: CustomScrollView(
          slivers: [
            const SliverSizedBox(height: 36),
            SliverSizedBox(
              height: 105,
              child: Container(
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.secondaryContainer,
                  borderRadius: BorderRadius.circular(8),
                ),
                alignment: Alignment.center,
                child: const Text('Placeholder image'),
              ),
            ),
            const SliverSizedBox(height: 24),
            SliverToBoxAdapter(
              child: Text(
                locale.walletPersonalizeDigidErrorPageTitle,
                textAlign: TextAlign.start,
                style: Theme.of(context).textTheme.displaySmall,
              ),
            ),
            const SliverSizedBox(height: 8),
            SliverToBoxAdapter(
              child: Text(
                locale.walletPersonalizeDigidErrorPageDescription,
                textAlign: TextAlign.start,
                style: Theme.of(context).textTheme.bodyLarge,
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
          onPressed: onRetryPressed,
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Image.asset(_kDigidLogoPath),
              const SizedBox(width: 12),
              Text(locale.walletPersonalizeDigidErrorPageLoginWithDigidCta),
            ],
          ),
        ),
        const SizedBox(height: 8),
        Center(
          child: TextIconButton(
            onPressed: onHelpPressed,
            child: Text(
              locale.walletPersonalizeDigidErrorPageNoDigidCta,
            ),
          ),
        ),
        const SizedBox(height: 24),
      ],
    );
  }
}
