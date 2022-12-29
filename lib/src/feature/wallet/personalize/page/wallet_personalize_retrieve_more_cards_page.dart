import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../common/widget/explanation_sheet.dart';
import '../../../common/widget/link_button.dart';
import '../../../common/widget/sliver_sized_box.dart';
import '../../../common/widget/text_icon_button.dart';

const _kMijnOverheidIllustration = 'assets/non-free/images/mijn_overheid_illustration.png';

class WalletPersonalizeRetrieveMoreCardsPage extends StatelessWidget {
  final VoidCallback onContinuePressed;
  final VoidCallback onSkipPressed;

  const WalletPersonalizeRetrieveMoreCardsPage({
    required this.onContinuePressed,
    required this.onSkipPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: Scrollbar(
        thumbVisibility: true,
        child: CustomScrollView(
          restorationId: 'check_data_offering_scrollview',
          slivers: <Widget>[
            const SliverSizedBox(height: 24),
            SliverToBoxAdapter(child: _buildHeaderSection(context)),
            const SliverSizedBox(height: 32),
            SliverToBoxAdapter(
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16),
                child: Image.asset(
                  _kMijnOverheidIllustration,
                  width: double.infinity,
                  fit: BoxFit.cover,
                ),
              ),
            ),
            SliverFillRemaining(
              hasScrollBody: false,
              fillOverscroll: true,
              child: _buildFooterSection(context),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            locale.walletPersonalizeRetrieveMoreCardsPageTitle,
            style: Theme.of(context).textTheme.headline2,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 8),
          Text(
            locale.walletPersonalizeRetrieveMoreCardsPageDescription,
            style: Theme.of(context).textTheme.bodyText1,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 16),
          LinkButton(
            customPadding: EdgeInsets.zero,
            child: Text(locale.walletPersonalizeRetrieveMoreCardsPageWhatIsRetrievedCta),
            onPressed: () {
              ExplanationSheet.show(
                context,
                title: locale.walletPersonalizeRetrieveMoreCardsPageInfoSheetTitle,
                description: locale.walletPersonalizeRetrieveMoreCardsPageInfoSheetDescription,
                closeButtonText: locale.walletPersonalizeRetrieveMoreCardsPageInfoSheetCloseCta,
              );
            },
          ),
        ],
      ),
    );
  }

  Widget _buildFooterSection(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      alignment: Alignment.bottomCenter,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          ElevatedButton(
            onPressed: onContinuePressed,
            child: Text(locale.walletPersonalizeRetrieveMoreCardsPageContinueCta),
          ),
          const SizedBox(height: 16),
          Center(
            child: TextIconButton(
              onPressed: onSkipPressed,
              child: Text(locale.walletPersonalizeRetrieveMoreCardsPageSkipCta),
            ),
          ),
        ],
      ),
    );
  }
}
