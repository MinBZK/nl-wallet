import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_staggered_grid_view/flutter_staggered_grid_view.dart';

import '../../../domain/model/wallet_card.dart';
import '../../../wallet_routes.dart';
import '../../common/widget/card/wallet_card_item.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/placeholder_screen.dart';
import 'bloc/card_overview_bloc.dart';

/// Defines the width required to render a card,
/// used to calculate the crossAxisCount.
const _kCardBreakPointWidth = 300.0;

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
    final crossAxisCount = max(1, (MediaQuery.of(context).size.width / _kCardBreakPointWidth).floor());
    return MasonryGridView.count(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      crossAxisCount: crossAxisCount,
      mainAxisSpacing: 16,
      crossAxisSpacing: 16,
      itemCount: cards.length + 1,
      itemBuilder: (context, index) {
        if (index < cards.length) {
          return _buildCardListItem(context, cards[index]);
        } else {
          return _buildFooterButton(context);
        }
      },
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
    return WalletCardItem.fromCardFront(
      front: walletCard.front,
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
