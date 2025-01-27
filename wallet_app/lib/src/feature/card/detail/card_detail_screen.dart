import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../../../domain/model/wallet_card.dart';
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
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/button/icon/help_icon_button.dart';
import '../../common/widget/card/wallet_card_item.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/info_row.dart';
import '../../common/widget/organization/organization_logo.dart';
import '../../common/widget/sliver_divider.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';
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
        child: Column(
          children: [
            Expanded(
              child: BlocBuilder<CardDetailBloc, CardDetailState>(
                builder: (context, state) {
                  return WalletScrollbar(
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
                  );
                },
              ),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  String _getTitle(BuildContext context, CardDetailState state) {
    final title = tryCast<CardDetailLoadSuccess>(state)?.detail.card.front.title.l10nValue(context);
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
                tag: card.id,
                flightShuttleBuilder: (
                  BuildContext flightContext,
                  Animation<double> animation,
                  HeroFlightDirection flightDirection,
                  BuildContext fromHeroContext,
                  BuildContext toHeroContext,
                ) {
                  animation
                      .addOnCompleteListener(() => context.read<CardDetailBloc>().notifyEntryTransitionCompleted());
                  return WalletCardItem.buildShuttleCard(animation, card.front, ctaAnimation: CtaAnimation.fadeOut);
                },
                child: WalletCardItem.fromCardFront(context: context, front: card.front),
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
          child: Semantics(
            image: true,
            attributedLabel: context.l10n.cardDetailScreenCardImageWCAGLabel(cardTitle).toAttributedString(context),
            excludeSemantics: true,
            child: FractionallySizedBox(
              widthFactor: 0.6,
              child: Hero(
                tag: card.id,
                flightShuttleBuilder: (
                  BuildContext flightContext,
                  Animation<double> animation,
                  HeroFlightDirection flightDirection,
                  BuildContext fromHeroContext,
                  BuildContext toHeroContext,
                ) =>
                    WalletCardItem.buildShuttleCard(animation, card.front, ctaAnimation: CtaAnimation.fadeIn),
                child: WalletCardItem.fromCardFront(context: context, front: card.front),
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
      InfoRow(
        icon: Icons.description_outlined,
        title: Text.rich(context.l10n.cardDetailScreenCardDataCta.toTextSpan(context)),
        onTap: () => _onCardDataPressed(context, card),
      ),
      InfoRow(
        icon: Icons.history_outlined,
        title: Text.rich(context.l10n.cardDetailScreenCardHistoryCta.toTextSpan(context)),
        subtitle: Text.rich(_createInteractionText(context, detail.mostRecentSuccessfulDisclosure).toTextSpan(context)),
        onTap: () => _onCardHistoryPressed(context, card.docType),
      ),
      InfoRow(
        leading: OrganizationLogo(image: card.issuer.logo, size: 24),
        title: Text.rich(context.l10n.cardDetailScreenIssuerCta.toTextSpan(context)),
        subtitle: Text.rich(card.issuer.displayName.l10nSpan(context)),
        onTap: () => OrganizationDetailScreen.showPreloaded(
          context,
          card.issuer,
          sharedDataWithOrganizationBefore: false,
          onReportIssuePressed: () => PlaceholderScreen.showGeneric(context),
        ),
      ),
      if (card.config.updatable)
        InfoRow(
          icon: Icons.replay_outlined,
          title: Text.rich(context.l10n.cardDetailScreenCardUpdateCta.toTextSpan(context)),
          subtitle: Text.rich(_createOperationText(context, detail.mostRecentIssuance).toTextSpan(context)),
          onTap: () => _onCardUpdatePressed(context, card),
        ),
      if (card.config.removable)
        InfoRow(
          icon: Icons.delete_outline_rounded,
          title: Text.rich(context.l10n.cardDetailScreenCardDeleteCta.toTextSpan(context)),
          onTap: () => _onCardDeletePressed(context),
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
      return context.l10n.cardDetailScreenLatestSuccessInteraction(
        attribute.relyingParty.displayName.l10nValue(context),
        status,
        timeAgo,
      );
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
            onPressed: () => context.read<CardDetailBloc>().add(CardDetailLoadTriggered(state.cardId)),
          ),
        ],
      ),
    );
  }

  void _onCardDataPressed(BuildContext context, WalletCard card) {
    Navigator.restorablePushNamed(
      context,
      WalletRoutes.cardDataRoute,
      arguments: CardDataScreenArgument(
        cardId: card.id,
        cardTitle: card.front.title.l10nValue(context),
      ).toMap(),
    );
  }

  void _onCardHistoryPressed(BuildContext context, String docType) {
    Navigator.pushNamed(
      context,
      WalletRoutes.cardHistoryRoute,
      arguments: docType,
    );
  }

  void _onCardUpdatePressed(BuildContext context, WalletCard card) {
    _showNoUpdateAvailableSheet(context);
  }

  void _onCardDeletePressed(BuildContext context) {
    PlaceholderScreen.showGeneric(context);
  }
}
