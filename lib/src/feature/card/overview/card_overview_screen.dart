import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/wallet_card.dart';
import '../../../wallet_routes.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../common/widget/wallet_card_front.dart';
import 'bloc/card_overview_bloc.dart';

class CardOverviewScreen extends StatelessWidget {
  const CardOverviewScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: _buildAppBar(context),
      body: _buildBody(context),
    );
  }

  PreferredSizeWidget _buildAppBar(BuildContext context) {
    return AppBar(
      leading: IconButton(
        icon: const Icon(Icons.lock_outline),
        onPressed: () => context.read<CardOverviewBloc>().add(CardOverviewLockWalletPressed()),
      ),
      title: Text(AppLocalizations.of(context).cardOverviewScreenTitle),
      actions: [
        IconButton(
          icon: const Icon(Icons.add),
          onPressed: () => _onCardCreatePressed(context),
        ),
      ],
    );
  }

  Widget _buildBody(BuildContext context) {
    return BlocBuilder<CardOverviewBloc, CardOverviewState>(
      builder: (context, state) {
        if (state is CardOverviewInitial) return _buildLoading();
        if (state is CardOverviewLoadInProgress) return _buildLoading();
        if (state is CardOverviewLoadSuccess) return _buildCards(context, state.cards);
        if (state is CardOverviewLoadFailure) return const Center(child: Icon(Icons.error_outline));
        throw UnsupportedError('Unknown state: $state');
      },
    );
  }

  Widget _buildLoading() {
    return const CenteredLoadingIndicator();
  }

  Widget _buildCards(BuildContext context, List<WalletCard> cards) {
    return Scrollbar(
      child: ListView.separated(
        padding: const EdgeInsets.fromLTRB(16.0, 24.0, 16.0, 24.0),
        itemCount: cards.length + 1,
        itemBuilder: (context, index) {
          if (index < cards.length) {
            return _buildCardListItem(context, cards[index]);
          } else {
            return _buildFooterButton(context);
          }
        },
        separatorBuilder: (context, index) => const SizedBox(height: 16.0),
      ),
    );
  }

  Widget _buildFooterButton(BuildContext context) {
    return TextButton(
      onPressed: () => _onCardCreatePressed(context),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.add),
          Text(AppLocalizations.of(context).cardOverviewScreenAddCardCta),
        ],
      ),
    );
  }

  Widget _buildCardListItem(BuildContext context, WalletCard walletCard) {
    return WalletCardFront(
      cardFront: walletCard.front,
      onPressed: () => _onCardPressed(context, walletCard.id),
    );
  }

  void _onCardCreatePressed(BuildContext context) {
    PlaceholderScreen.show(context);
  }

  void _onCardPressed(BuildContext context, String cardId) {
    Navigator.restorablePushNamed(context, WalletRoutes.cardSummaryRoute, arguments: cardId);
  }
}
