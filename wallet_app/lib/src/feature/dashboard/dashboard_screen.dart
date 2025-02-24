import 'dart:math';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_staggered_grid_view/flutter_staggered_grid_view.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:visibility_detector/visibility_detector.dart';

import '../../data/service/navigation_service.dart';
import '../../domain/model/wallet_card.dart';
import '../../navigation/secured_page_route.dart';
import '../../navigation/wallet_routes.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../wallet_assets.dart';
import '../card/detail/argument/card_detail_screen_argument.dart';
import '../card/detail/card_detail_screen.dart';
import '../common/widget/activity_summary.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/icon/menu_icon_text_button.dart';
import '../common/widget/button/icon/qr_icon_button.dart';
import '../common/widget/card/wallet_card_item.dart';
import '../common/widget/centered_loading_indicator.dart';
import '../common/widget/fade_in_at_offset.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/text_with_link.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import '../update/widget/update_banner.dart';
import 'argument/dashboard_screen_argument.dart';
import 'bloc/dashboard_bloc.dart';

/// Defines the width required to render a card,
/// used to calculate the crossAxisCount.
const _kCardBreakPointWidth = 300.0;
const _kMaxCrossAxisCount = 2;

class DashboardScreen extends StatelessWidget {
  static DashboardScreenArgument? getArgument(RouteSettings settings) {
    final args = settings.arguments;
    if (args == null) return null;
    try {
      return DashboardScreenArgument.fromJson(args as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      return null;
    }
  }

  const DashboardScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('dashboardScreen'),
      appBar: _buildAppBar(context),
      body: VisibilityDetector(
        key: const Key('dashboardVisibilityDetector'),
        onVisibilityChanged: (visibilityInfo) {
          if (visibilityInfo.visibleFraction >= 1) {
            context.read<NavigationService>().processQueue();
            SemanticsService.announce(context.l10n.dashboardScreenOverviewAnnouncement, TextDirection.ltr);
          }
        },
        child: _buildBody(context),
      ),
    );
  }

  PreferredSizeWidget _buildAppBar(BuildContext context) {
    return WalletAppBar(
      automaticallyImplyLeading: false,
      leading: _buildLeadingMenuButton(context),
      leadingWidth: double.infinity,
      actions: const [
        FadeInAtOffset(
          visibleOffset: 150,
          appearOffset: 100,
          child: QrIconButton(),
        ),
        HelpIconButton(),
      ],
    );
  }

  /// Builds the menu button, wrapped in [Align] and [Semantics] to make sure the
  /// correct info and FocusArea is provided for TalkBack/VoiceOver.
  Widget _buildLeadingMenuButton(BuildContext context) {
    return Align(
      alignment: Alignment.centerLeft,
      child: MenuIconTextButton(
        onPressed: () => Navigator.pushNamed(context, WalletRoutes.menuRoute),
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    return SafeArea(
      bottom: false,
      top: false,
      child: BlocBuilder<DashboardBloc, DashboardState>(
        builder: (context, state) {
          return switch (state) {
            DashboardStateInitial() => _buildLoading(),
            DashboardLoadInProgress() => _buildLoading(),
            DashboardLoadSuccess() => _buildContent(context, state),
            DashboardLoadFailure() => _buildError(context),
          };
        },
      ),
    );
  }

  Widget _buildLoading() {
    return const CenteredLoadingIndicator();
  }

  Widget _buildContent(BuildContext context, DashboardLoadSuccess state) {
    return WalletScrollbar(
      child: CustomScrollView(
        slivers: [
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
            sliver: SliverToBoxAdapter(
              child: StreamBuilder<bool>(
                stream: context.read<NavigationService>().observeUpdateNotificationDialogVisible(),
                builder: (context, snapshot) {
                  // Only show the update banner when stream returns data and the dialog is not visible.
                  final dialogVisibleData = snapshot.data;
                  return (dialogVisibleData != null && dialogVisibleData)
                      ? const SizedBox.shrink()
                      : const UpdateBanner();
                },
              ),
            ),
          ),
          SliverToBoxAdapter(
            child: Container(
              height: 250,
              alignment: Alignment.center,
              child: _buildQrLogo(context),
            ),
          ),
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
            sliver: SliverToBoxAdapter(
              child: ActivitySummary(
                events: state.history ?? [],
                onTap: () => Navigator.pushNamed(context, WalletRoutes.walletHistoryRoute),
              ),
            ),
          ),
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
            sliver: _buildCardsSliver(context, state.cards),
          ),
          SliverToBoxAdapter(
            child: _buildFooter(context),
          ),
          SliverSizedBox(
            height: context.mediaQuery.padding.bottom,
          ),
        ],
      ),
    );
  }

  Widget _buildCardsSliver(BuildContext context, List<WalletCard> cards) {
    final crossAxisCount = max(1, (context.mediaQuery.size.width / _kCardBreakPointWidth).floor());
    return SliverMasonryGrid(
      gridDelegate: SliverSimpleGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: min(crossAxisCount, _kMaxCrossAxisCount),
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
        linkText: cta,
        fullText: fullString,
        textAlign: TextAlign.center,
        onLinkPressed: () => Navigator.pushNamed(context, WalletRoutes.aboutRoute),
        style: context.textTheme.bodyMedium,
      ),
    );
  }

  Widget _buildQrLogo(BuildContext context) {
    onTapQr() => Navigator.pushNamed(context, WalletRoutes.qrRoute);
    return Semantics(
      attributedLabel: context.l10n.dashboardScreenQrCta.toAttributedString(context),
      button: true,
      excludeSemantics: true,
      child: SizedBox(
        width: context.mediaQuery.size.width * 0.6,
        height: 240,
        child: GestureDetector(
          onTap: onTapQr,
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            mainAxisSize: MainAxisSize.min,
            children: [
              TextButton(
                onPressed: onTapQr,
                child: Column(
                  children: [
                    SvgPicture.asset(WalletAssets.svg_qr_button),
                    const SizedBox(height: 16),
                    Text(
                      context.l10n.dashboardScreenQrCta,
                      textAlign: TextAlign.center,
                    ),
                    const SizedBox(height: 24),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildCardListItem(BuildContext context, WalletCard walletCard) {
    return Hero(
      tag: walletCard.id,
      child: WalletCardItem.fromCardFront(
        context: context,
        key: Key(walletCard.docType),
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
    return SafeArea(
      minimum: const EdgeInsets.only(left: 16, right: 16, bottom: 16),
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
            child: Text.rich(context.l10n.generalRetry.toTextSpan(context)),
          ),
        ],
      ),
    );
  }

  /// Show the [DashboardScreen], placing it at the root of the navigation stack. When [cards] are provided
  /// the [DashboardBloc] is initialized with these cards, so that they are instantly
  /// available, e.g. useful when triggering Hero animations.
  static void show(BuildContext context, {List<WalletCard>? cards}) {
    if (cards != null) SecuredPageRoute.overrideDurationOfNextTransition(const Duration(milliseconds: 1200));
    Navigator.restorablePushNamedAndRemoveUntil(
      context,
      WalletRoutes.dashboardRoute,
      ModalRoute.withName(WalletRoutes.splashRoute),
      arguments: cards == null ? null : DashboardScreenArgument(cards: cards).toJson(),
    );
  }
}
