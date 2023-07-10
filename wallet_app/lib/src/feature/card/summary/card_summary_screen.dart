import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../domain/model/wallet_card_summary.dart';
import '../../../domain/usecase/card/get_wallet_card_update_issuance_request_id_usecase.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/formatter/card_valid_until_time_formatter.dart';
import '../../../util/formatter/operation_issued_time_formatter.dart';
import '../../../util/formatter/time_ago_formatter.dart';
import '../../../util/mapper/timeline_attribute_status_mapper.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/card/wallet_card_item.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/explanation_sheet.dart';
import '../../common/widget/info_row.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../issuance/argument/issuance_screen_argument.dart';
import '../data/argument/card_data_screen_argument.dart';
import 'argument/card_summary_screen_argument.dart';
import 'bloc/card_summary_bloc.dart';

const _kCardExpiresInDays = 365; // 1 year for demo purposes

class CardSummaryScreen extends StatelessWidget {
  static CardSummaryScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return CardSummaryScreenArgument.fromMap(args as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in [CardSummaryScreenArgument] when opening the CardSummaryScreen');
    }
  }

  final String cardTitle;

  const CardSummaryScreen({required this.cardTitle, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(cardTitle),
      ),
      body: SafeArea(
        child: _buildBody(context),
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    return BlocBuilder<CardSummaryBloc, CardSummaryState>(
      builder: (context, state) {
        return switch (state) {
          CardSummaryInitial() => _buildLoading(),
          CardSummaryLoadInProgress() => _buildLoading(),
          CardSummaryLoadSuccess() => _buildSummary(context, state.summary),
          CardSummaryLoadFailure() => _buildError(context, state),
        };
      },
    );
  }

  Widget _buildLoading() => const CenteredLoadingIndicator();

  Widget _buildSummary(BuildContext context, WalletCardSummary summary) {
    final card = summary.card;

    return Column(
      children: [
        Expanded(
          child: Scrollbar(
            child: ListView(
              padding: const EdgeInsets.only(top: 24),
              children: [
                const SizedBox(height: 8),
                ExcludeSemantics(
                  child: FractionallySizedBox(
                    widthFactor: 0.6,
                    child: WalletCardItem.fromCardFront(front: card.front),
                  ),
                ),
                const SizedBox(height: 32),
                const Divider(height: 1),
                InfoRow(
                  icon: Icons.description_outlined,
                  title: Text(context.l10n.cardSummaryScreenCardDataCta),
                  subtitle: Text(context.l10n.cardSummaryScreenCardDataIssuedBy(summary.issuer.shortName)),
                  onTap: () => _onCardDataPressed(context, card),
                ),
                const Divider(height: 1),
                InfoRow(
                  icon: Icons.history_outlined,
                  title: Text(context.l10n.cardSummaryScreenCardHistoryCta),
                  subtitle: Text(_createInteractionText(context, summary.latestSuccessInteraction)),
                  onTap: () => _onCardHistoryPressed(context, card.id),
                ),
                const Divider(height: 1),
                InfoRow(
                  icon: Icons.replay_outlined,
                  title: Text(context.l10n.cardSummaryScreenCardUpdateCta),
                  subtitle: Text(_createOperationText(context, summary.latestIssuedOperation)),
                  onTap: () => _onCardUpdatePressed(context, card),
                ),
                const Divider(height: 1),
                if (card.config.removable) ...[
                  InfoRow(
                    icon: Icons.delete_outline_rounded,
                    title: Text(context.l10n.cardSummaryScreenCardDeleteCta),
                    onTap: () => _onCardDeletePressed(context),
                  ),
                  const Divider(height: 1)
                ],
              ],
            ),
          ),
        ),
        const BottomBackButton(showDivider: true),
      ],
    );
  }

  void _showNoUpdateAvailableSheet(BuildContext context) {
    ExplanationSheet.show(
      context,
      title: context.l10n.cardSummaryScreenNoUpdateAvailableSheetTitle,
      description: context.l10n.cardSummaryScreenNoUpdateAvailableSheetDescription,
      closeButtonText: context.l10n.cardSummaryScreenNoUpdateAvailableSheetCloseCta,
    );
  }

  String _createInteractionText(BuildContext context, InteractionTimelineAttribute? attribute) {
    if (attribute != null) {
      final String timeAgo = TimeAgoFormatter.format(context, attribute.dateTime);
      final String status = TimelineAttributeStatusTextMapper.map(context, attribute).toLowerCase();
      return context.l10n.cardSummaryScreenLatestSuccessInteraction(
        attribute.organization.shortName,
        status,
        timeAgo,
      );
    } else {
      return context.l10n.cardSummaryScreenLatestSuccessInteractionUnknown;
    }
  }

  String _createOperationText(BuildContext context, OperationTimelineAttribute? attribute) {
    if (attribute != null) {
      DateTime issued = attribute.dateTime;
      String issuedTime = OperationIssuedTimeFormatter.format(context, issued);
      String issuedText = context.l10n.cardSummaryScreenLatestIssuedOperation(issuedTime);

      DateTime validUntil = issued.add(const Duration(days: _kCardExpiresInDays));
      String validUntilTime = CardValidUntilTimeFormatter.format(context, validUntil);
      String validUntilText = context.l10n.cardSummaryScreenCardValidUntil(validUntilTime);

      return '$issuedText\n$validUntilText';
    } else {
      return context.l10n.cardSummaryScreenLatestIssuedOperationUnknown;
    }
  }

  Widget _buildError(BuildContext context, CardSummaryLoadFailure state) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.error_outline),
          const SizedBox(height: 16),
          TextButton(
            child: Text(context.l10n.generalRetry),
            onPressed: () => context.read<CardSummaryBloc>().add(CardSummaryLoadTriggered(state.cardId)),
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
        cardTitle: card.front.title,
      ).toMap(),
    );
  }

  void _onCardHistoryPressed(BuildContext context, String cardId) {
    Navigator.restorablePushNamed(context, WalletRoutes.cardHistoryRoute, arguments: cardId);
  }

  /// Temporary async logic inside [CardSummaryScreen] class;
  /// This async flow isn't designed (happy & unhappy paths); it's to do for after demo day.
  void _onCardUpdatePressed(BuildContext context, WalletCard card) {
    GetWalletCardUpdateIssuanceRequestIdUseCase useCase = context.read();
    useCase.invoke(card).then((issuanceRequestId) {
      if (issuanceRequestId != null) {
        Navigator.restorablePushNamed(
          context,
          WalletRoutes.issuanceRoute,
          arguments: IssuanceScreenArgument(
            sessionId: issuanceRequestId,
            isRefreshFlow: true,
          ).toMap(),
        );
      } else {
        _showNoUpdateAvailableSheet(context);
      }
    });
  }

  void _onCardDeletePressed(BuildContext context) {
    PlaceholderScreen.show(context);
  }
}
