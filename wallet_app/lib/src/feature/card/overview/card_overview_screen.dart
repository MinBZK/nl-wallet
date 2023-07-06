import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_staggered_grid_view/flutter_staggered_grid_view.dart';

import '../../../domain/model/wallet_card.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/card/wallet_card_item.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../summary/argument/card_summary_screen_argument.dart';
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
      title: Text(context.l10n.cardOverviewScreenTitle),
    );
  }

  Widget _buildBody(BuildContext context) {
    return BlocBuilder<CardOverviewBloc, CardOverviewState>(
      builder: (context, state) {
        return switch (state) {
          CardOverviewInitial() => _buildLoading(),
          CardOverviewLoadInProgress() => _buildLoading(),
          CardOverviewLoadSuccess() => _buildCards(context, state.cards),
          CardOverviewLoadFailure() => _buildError(context),
        };
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
      itemCount: cards.length,
      itemBuilder: (context, index) => _buildCardListItem(context, cards[index]),
    );
  }

  Widget _buildCardListItem(BuildContext context, WalletCard walletCard) {
    return WalletCardItem.fromCardFront(
      front: walletCard.front,
      onPressed: () => _onCardPressed(context, walletCard),
    );
  }

  void _onCardPressed(BuildContext context, WalletCard walletCard) {
    Navigator.restorablePushNamed(
      context,
      WalletRoutes.cardSummaryRoute,
      arguments: CardSummaryScreenArgument(
        cardId: walletCard.id,
        cardTitle: walletCard.front.title,
      ).toMap(),
    );
  }

  Widget _buildError(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          const Spacer(),
          Text(
            context.l10n.errorScreenGenericDescription,
            textAlign: TextAlign.center,
          ),
          const Spacer(),
          ElevatedButton(
            onPressed: () => context.read<CardOverviewBloc>().add(CardOverviewLoadTriggered()),
            child: Text(context.l10n.generalRetry),
          ),
        ],
      ),
    );
  }
}
