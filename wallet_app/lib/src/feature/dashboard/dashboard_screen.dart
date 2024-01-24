import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_staggered_grid_view/flutter_staggered_grid_view.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../../domain/model/wallet_card.dart';
import '../../navigation/secured_page_route.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_assets.dart';
import '../card/detail/argument/card_detail_screen_argument.dart';
import '../card/detail/card_detail_screen.dart';
import '../common/screen/placeholder_screen.dart';
import '../common/widget/card/wallet_card_item.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/fade_in_at_offset.dart';
import '../common/widget/text_with_link.dart';
import '../common/widget/wallet_app_bar.dart';
import '../home/bloc/home_bloc.dart';
import 'bloc/dashboard_bloc.dart';

/// Defines the width required to render a card,
/// used to calculate the crossAxisCount.
const _kCardBreakPointWidth = 300.0;

class DashboardScreen extends StatelessWidget {
  const DashboardScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('cardOverviewScreen'),
      appBar: _buildAppBar(context),
      body: _buildBody(context),
    );
  }

  PreferredSizeWidget _buildAppBar(BuildContext context) {
    return WalletAppBar(
      leading: IconButton(
        onPressed: () => context.read<HomeBloc>().add(const HomeTabPressed(HomeTab.menu)),
        icon: const Icon(Icons.menu),
      ),
      title: Text(
        context.l10n.dashboardScreenTitle,
        style: context.textTheme.titleMedium!.copyWith(color: context.colorScheme.primary, fontWeight: FontWeight.bold),
      ),
      actions: [
        FadeInAtOffset(
            visibleOffset: 150,
            appearOffset: 100,
            child: IconButton(
              onPressed: () => context.read<HomeBloc>().add(const HomeTabPressed(HomeTab.qr)),
              icon: const Icon(Icons.qr_code_rounded),
            )),
        IconButton(
          onPressed: () => PlaceholderScreen.show(context),
          icon: const Icon(Icons.help_outline_rounded),
        ),
      ],
    );
  }

  Widget _buildBody(BuildContext context) {
    return BlocBuilder<DashboardBloc, DashboardState>(
      builder: (context, state) {
        return switch (state) {
          DashboardStateInitial() => _buildLoading(),
          DashboardLoadInProgress() => _buildLoading(),
          DashboardLoadSuccess() => _buildContent(context, state),
          DashboardLoadFailure() => _buildError(context),
        };
      },
    );
  }

  Widget _buildLoading() {
    return const CenteredLoadingIndicator();
  }

  Widget _buildContent(BuildContext context, DashboardLoadSuccess state) {
    return CustomScrollView(
      slivers: [
        SliverToBoxAdapter(
          child: Container(
            height: 250,
            alignment: Alignment.center,
            child: _buildQrLogo(context),
          ),
        ),
        SliverPadding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
          sliver: _buildCardsSliver(context, state.cards),
        ),
        SliverToBoxAdapter(
          child: _buildFooter(context),
        )
      ],
    );
  }

  Widget _buildCardsSliver(BuildContext context, List<WalletCard> cards) {
    final crossAxisCount = max(1, (context.mediaQuery.size.width / _kCardBreakPointWidth).floor());
    return SliverMasonryGrid(
      gridDelegate: SliverSimpleGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: crossAxisCount,
      ),
      mainAxisSpacing: 16,
      crossAxisSpacing: 16,
      delegate: SliverChildBuilderDelegate(
        (context, index) => _buildCardListItem(context, cards[index]),
        childCount: cards.length,
      ),
    );
  }

  Widget _buildFooter(BuildContext context) {
    final cta = context.l10n.dashboardScreenFooterCta;
    final fullString = context.l10n.dashboardScreenFooter(cta);
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 40, horizontal: 24),
      child: TextWithLink(
        ctaText: cta,
        fullText: fullString,
        textAlign: TextAlign.center,
        onCtaPressed: () => Navigator.pushNamed(context, WalletRoutes.aboutRoute),
        style: context.textTheme.bodyMedium,
      ),
    );
  }

  Widget _buildQrLogo(BuildContext context) {
    onTapQr() => context.read<HomeBloc>().add(const HomeTabPressed(HomeTab.qr));
    return GestureDetector(
      onTap: onTapQr,
      child: MergeSemantics(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            SvgPicture.asset(WalletAssets.svg_qr_button),
            TextButton(
              onPressed: onTapQr,
              child: Text(context.l10n.dashboardScreenQrCta),
            )
          ],
        ),
      ),
    );
  }

  Widget _buildCardListItem(BuildContext context, WalletCard walletCard) {
    return Hero(
      tag: walletCard.id,
      child: WalletCardItem.fromCardFront(
        context: context,
        front: walletCard.front,
        onPressed: () => _onCardPressed(context, walletCard),
      ),
    );
  }

  void _onCardPressed(BuildContext context, WalletCard walletCard) {
    SecuredPageRoute.overrideDurationOfNextTransition(kPreferredCardDetailEntryTransitionDuration);
    Navigator.restorablePushNamed(
      context,
      WalletRoutes.cardDetailRoute,
      arguments: CardDetailScreenArgument.forCard(walletCard).toJson(),
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
            onPressed: () => context.read<DashboardBloc>().add(const DashboardLoadTriggered()),
            child: Text(context.l10n.generalRetry),
          ),
        ],
      ),
    );
  }
}
