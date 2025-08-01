import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/event/wallet_event.dart';
import '../../../domain/model/wallet_card_detail.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/cast_util.dart';
import '../../../util/extension/animation_extension.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/localized_text_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../util/formatter/card_valid_until_time_formatter.dart';
import '../../../util/formatter/operation_issued_time_formatter.dart';
import '../../../util/formatter/time_ago_formatter.dart';
import '../../../util/mapper/event/wallet_event_status_text_mapper.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/sheet/explanation_sheet.dart';
import '../../common/widget/animated_fade_in.dart';
import '../../common/widget/button/button_content.dart';
import '../../common/widget/button/icon/help_icon_button.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/card/wallet_card_item.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/menu_item.dart';
import '../../common/widget/organization/organization_logo.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';
import '../../common/widget/spacer/sliver_divider.dart';
import '../../common/widget/spacer/sliver_sized_box.dart';
import '../../common/widget/wallet_scrollbar.dart';
import '../../organization/detail/organization_detail_screen.dart';
import '../data/argument/card_data_screen_argument.dart';
import 'argument/card_detail_screen_argument.dart';
import 'bloc/card_detail_bloc.dart';

/// This value can be used with [SecuredPageRoute.overrideDurationOfNextTransition] when navigating to the
/// [CardDetailScreen] to slow down the entry transition a bit, making it feel a bit less rushed when the card
/// animates into place.
const kPreferredCardDetailEntryTransitionDuration = Duration(milliseconds: 600);
const _kCardExpiresInDays = 365; // 1 year for demo purposes

class CardDetailScreen extends StatelessWidget {
  static CardDetailScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return CardDetailScreenArgument.fromJson(args! as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode type: ${args.runtimeType} arg: $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError(
        'Make sure to pass in [CardDetailScreenArgument] as json when opening the CardDetailScreen',
      );
    }
  }

  final String cardTitle;

  const CardDetailScreen({required this.cardTitle, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('cardDetailScreen'),
      body: SafeArea(
        child: BlocBuilder<CardDetailBloc, CardDetailState>(
          builder: (context, state) {
            return Column(
              children: [
                Expanded(
                  child: WalletScrollbar(
                    child: CustomScrollView(
                      slivers: [
                        SliverWalletAppBar(
                          title: _getTitle(context, state),
                          scrollController: PrimaryScrollController.maybeOf(context),
                          actions: const [HelpIconButton()],
                        ),
                        _buildBody(context, state),
                      ],
                    ),
                  ),
                ),
                _buildBottomSection(context, state),
              ],
            );
          },
        ),
      ),
    );
  }

  String _getTitle(BuildContext context, CardDetailState state) {
    final title = tryCast<CardDetailLoadSuccess>(state)?.detail.card.title.l10nValue(context);
    return title ?? cardTitle;
  }

  Widget _buildBody(BuildContext context, CardDetailState state) {
    return switch (state) {
      CardDetailInitial() => _buildLoading(context),
      CardDetailLoadInProgress() => _buildLoading(context, card: state.card),
      CardDetailLoadSuccess() => _buildDetail(context, state.detail),
      CardDetailLoadFailure() => _buildError(context, state),
    };
  }

  Widget _buildLoading(BuildContext context, {WalletCard? card}) {
    if (card == null) {
      return const SliverFillRemaining(
        hasScrollBody: false,
        child: CenteredLoadingIndicator(),
      );
    }
    return SliverMainAxisGroup(
      slivers: [
        const SliverSizedBox(height: 24 + 8),
        SliverToBoxAdapter(
          child: ExcludeSemantics(
            child: FractionallySizedBox(
              widthFactor: 0.6,
              child: Hero(
                tag: card.hashCode,
                flightShuttleBuilder: (
                  BuildContext flightContext,
                  Animation<double> animation,
                  HeroFlightDirection flightDirection,
                  BuildContext fromHeroContext,
                  BuildContext toHeroContext,
                ) {
                  animation
                      .addOnCompleteListener(() => context.read<CardDetailBloc>().notifyEntryTransitionCompleted());
                  return WalletCardItem.buildShuttleCard(animation, card, ctaAnimation: CtaAnimation.fadeOut);
                },
                child: WalletCardItem.fromWalletCard(context, card),
              ),
            ),
          ),
        ),
        const SliverSizedBox(height: 32),
        const SliverDivider(),
        const SliverFillRemaining(
          child: CenteredLoadingIndicator(),
        ),
      ],
    );
  }

  Widget _buildDetail(BuildContext context, WalletCardDetail detail) {
    final card = detail.card;

    return SliverMainAxisGroup(
      slivers: [
        const SliverSizedBox(height: 24 + 8),
        SliverToBoxAdapter(
          child: ExcludeSemantics(
            child: FractionallySizedBox(
              widthFactor: 0.6,
              child: Hero(
                tag: card.hashCode,
                flightShuttleBuilder: (
                  BuildContext flightContext,
                  Animation<double> animation,
                  HeroFlightDirection flightDirection,
                  BuildContext fromHeroContext,
                  BuildContext toHeroContext,
                ) =>
                    WalletCardItem.buildShuttleCard(animation, card, ctaAnimation: CtaAnimation.fadeIn),
                child: WalletCardItem.fromWalletCard(context, card),
              ),
            ),
          ),
        ),
        const SliverSizedBox(height: 32),
        const SliverDivider(),
        SliverToBoxAdapter(
          child: AnimatedFadeIn(
            child: _buildDetailContent(context, detail),
          ),
        ),
        const SliverDivider(),
        const SliverSizedBox(height: 24),
      ],
    );
  }

  Widget _buildDetailContent(BuildContext context, WalletCardDetail detail) {
    final card = detail.card;
    final rows = [
      MenuItem(
        leftIcon: const Icon(Icons.description_outlined),
        label: Text.rich(context.l10n.cardDetailScreenCardDataCta.toTextSpan(context)),
        subtitle: Text.rich(context.l10n.cardDetailScreenCardDataPrivacyWarning.toTextSpan(context)),
        onPressed: () => _onCardDataPressed(context, card),
      ),
      MenuItem(
        leftIcon: const Icon(Icons.history_outlined),
        label: Text.rich(context.l10n.cardDetailScreenCardHistoryCta.toTextSpan(context)),
        subtitle: Text.rich(_createInteractionText(context, detail.mostRecentSuccessfulDisclosure).toTextSpan(context)),
        onPressed: card.attestationId == null ? null : () => _onCardHistoryPressed(context, card.attestationId!),
      ),
      MenuItem(
        leftIcon: OrganizationLogo(image: card.issuer.logo, size: 24),
        label: Text.rich(context.l10n.cardDetailScreenIssuerCta.toTextSpan(context)),
        subtitle: Text.rich(card.issuer.displayName.l10nSpan(context)),
        onPressed: () => OrganizationDetailScreen.showPreloaded(
          context,
          card.issuer,
          sharedDataWithOrganizationBefore: false,
          onReportIssuePressed: () => PlaceholderScreen.showGeneric(context),
        ),
      ),
      if (card.config.updatable)
        MenuItem(
          leftIcon: const Icon(Icons.replay_outlined),
          label: Text.rich(context.l10n.cardDetailScreenCardUpdateCta.toTextSpan(context)),
          subtitle: Text.rich(_createOperationText(context, detail.mostRecentIssuance).toTextSpan(context)),
          onPressed: () => _onCardUpdatePressed(context, card),
        ),
      if (card.config.removable)
        MenuItem(
          leftIcon: const Icon(Icons.delete_outline_rounded),
          label: Text.rich(context.l10n.cardDetailScreenCardDeleteCta.toTextSpan(context)),
          onPressed: () => _onCardDeletePressed(context),
        ),
    ];
    return ListView.separated(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemBuilder: (c, i) => rows[i],
      separatorBuilder: (c, i) => const Divider(),
      itemCount: rows.length,
    );
  }

  void _showNoUpdateAvailableSheet(BuildContext context) {
    ExplanationSheet.show(
      context,
      title: context.l10n.cardDetailScreenNoUpdateAvailableSheetTitle,
      description: context.l10n.cardDetailScreenNoUpdateAvailableSheetDescription,
      closeButtonText: context.l10n.cardDetailScreenNoUpdateAvailableSheetCloseCta,
    );
  }

  String _createInteractionText(BuildContext context, DisclosureEvent? attribute) {
    if (attribute != null) {
      final String timeAgo = TimeAgoFormatter.format(context, attribute.dateTime);
      final String status = WalletEventStatusTextMapper().map(context, attribute).toLowerCase();
      return context.l10n
          .cardDetailScreenLatestSuccessInteraction(
            attribute.relyingParty.displayName.l10nValue(context),
            status,
            timeAgo,
          )
          .capitalize;
    } else {
      return context.l10n.cardDetailScreenLatestSuccessInteractionUnknown;
    }
  }

  String _createOperationText(BuildContext context, IssuanceEvent? event) {
    if (event == null) return context.l10n.cardDetailScreenLatestIssuedOperationUnknown;

    final String issuedTime = OperationIssuedTimeFormatter.format(context, event.dateTime);
    final String issuedText = context.l10n.cardDetailScreenLatestIssuedOperation(issuedTime);

    // TODO(anyone): Don't hardcode expiry
    final DateTime validUntil = event.dateTime.add(const Duration(days: _kCardExpiresInDays));
    final String validUntilTime = CardValidUntilTimeFormatter.format(context, validUntil);
    final String validUntilText = context.l10n.cardDetailScreenCardValidUntil(validUntilTime);

    return '$issuedText\n$validUntilText';
  }

  Widget _buildError(BuildContext context, CardDetailLoadFailure state) {
    return SliverFillRemaining(
      hasScrollBody: false,
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        mainAxisSize: MainAxisSize.max,
        children: [
          const Icon(Icons.error_outline),
          const SizedBox(height: 16),
          TextButton(
            child: Text.rich(context.l10n.generalRetry.toTextSpan(context)),
            onPressed: () => context.read<CardDetailBloc>().add(CardDetailLoadTriggered(state.attestationId)),
          ),
        ],
      ),
    );
  }

  void _onCardDataPressed(BuildContext context, WalletCard card) {
    assert(
      card.isPersisted,
      'To view the data, the card should be persisted and thus have an id. Otherwise the CardDataScreen will render an error.',
    );
    Navigator.restorablePushNamed(
      context,
      WalletRoutes.cardDataRoute,
      arguments: CardDataScreenArgument(
        cardId: card.attestationId ?? '',
        cardTitle: card.title.l10nValue(context),
      ).toMap(),
    );
  }

  void _onCardHistoryPressed(BuildContext context, String attestationId) {
    Navigator.pushNamed(
      context,
      WalletRoutes.cardHistoryRoute,
      arguments: attestationId,
    );
  }

  void _onCardUpdatePressed(BuildContext context, WalletCard card) {
    _showNoUpdateAvailableSheet(context);
  }

  void _onCardDeletePressed(BuildContext context) {
    PlaceholderScreen.showGeneric(context);
  }

  Widget _buildBottomSection(BuildContext context, CardDetailState state) {
    final showRefreshButton = tryCast<CardDetailLoadSuccess>(state)?.showRenewOption ?? false;
    return Column(
      mainAxisSize: MainAxisSize.max,
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        const Divider(),
        if (showRefreshButton)
          Padding(
            padding: const EdgeInsets.only(left: 16, right: 16, top: 24, bottom: 12),
            child: PrimaryButton(
              text: Text.rich(context.l10n.cardDetailScreenRenewPidCta.toTextSpan(context)),
              icon: const Icon(Icons.arrow_forward_outlined),
              onPressed: () => Navigator.of(context).pushNamed(WalletRoutes.renewPidRoute),
            ),
          ),
        ListButton(
          onPressed: () => Navigator.maybePop(context),
          icon: const Icon(Icons.arrow_back),
          mainAxisAlignment: MainAxisAlignment.center,
          iconPosition: IconPosition.start,
          dividerSide: DividerSide.none,
          text: Text.rich(context.l10n.generalBottomBackCta.toTextSpan(context)),
        ),
      ],
    );
  }
}
