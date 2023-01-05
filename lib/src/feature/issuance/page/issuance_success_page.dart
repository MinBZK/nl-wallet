import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/card_front.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../common/widget/stacked_wallet_cards.dart';
import '../../common/widget/status_icon.dart';

class IssuanceSuccessPage extends StatelessWidget {
  final VoidCallback onClose;
  final List<CardFront> cards;
  final bool isRefreshFlow;

  const IssuanceSuccessPage({
    required this.onClose,
    required this.cards,
    required this.isRefreshFlow,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      thumbVisibility: true,
      child: CustomScrollView(
        controller: ScrollController(),
        restorationId: 'issuance_success_page',
        slivers: <Widget>[
          const SliverSizedBox(height: 48.0),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          const SliverSizedBox(height: 32.0),
          SliverToBoxAdapter(
              child: Column(
            children: [
              const SizedBox(height: 16),
              Padding(
                padding: const EdgeInsets.all(16),
                child: StackedWalletCards(cards: cards),
              ),
            ],
          )),
          const SliverSizedBox(height: 16.0),
          SliverFillRemaining(hasScrollBody: false, fillOverscroll: true, child: _buildBottomSection(context)),
        ],
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    final locale = AppLocalizations.of(context);
    final subtitle = isRefreshFlow
        ? locale.issuanceSuccessPageCardsUpdatedSubtitle(cards.length)
        : locale.issuanceSuccessPageCardsAddedSubtitle(cards.length);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16.0),
          child: StatusIcon(
            icon: Icons.check,
            color: Theme.of(context).primaryColor,
          ),
        ),
        const SizedBox(height: 32.0),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16.0),
          child: Text(
            locale.issuanceSuccessPageTitle,
            style: Theme.of(context).textTheme.headline2,
            textAlign: TextAlign.center,
          ),
        ),
        const SizedBox(height: 8.0),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16.0),
          child: Text(
            subtitle,
            style: Theme.of(context).textTheme.bodyText1,
            textAlign: TextAlign.center,
          ),
        ),
      ],
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Align(
      alignment: Alignment.bottomCenter,
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: SizedBox(
          height: 48,
          child: ElevatedButton(
            onPressed: onClose,
            child: Text(AppLocalizations.of(context).issuanceSuccessPageCloseCta),
          ),
        ),
      ),
    );
  }
}
