import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../domain/model/wallet_card_summary.dart';
import '../../../domain/usecase/card/get_wallet_card_update_issuance_request_id_usecase.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/formatter/card_valid_until_time_formatter.dart';
import '../../../util/formatter/operation_issued_time_formatter.dart';
import '../../../util/formatter/time_ago_formatter.dart';
import '../../../util/mapper/timeline_attribute_status_mapper.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/card/sized_card_front.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/explanation_sheet.dart';
import '../../common/widget/info_row.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../issuance/argument/issuance_screen_argument.dart';
import '../data/argument/card_data_screen_argument.dart';
import 'argument/card_summary_screen_argument.dart';
import 'bloc/card_summary_bloc.dart';

const _kCardExpiresInDays = 365; // 1 year for demo purposes
const _kCardDisplayPaddingHorizontal = 56;

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
        if (state is CardSummaryInitial) return _buildLoading();
        if (state is CardSummaryLoadInProgress) return _buildLoading();
        if (state is CardSummaryLoadSuccess) return _buildSummary(context, state.summary);
        if (state is CardSummaryLoadFailure) return _buildError(context, state);
        throw UnsupportedError('Unknown state: $state');
      },
    );
  }

  Widget _buildLoading() => const CenteredLoadingIndicator();

  Widget _buildSummary(BuildContext context, WalletCardSummary summary) {
    final locale = AppLocalizations.of(context);
    final card = summary.card;

    return Column(
      children: [
        Expanded(
          child: Scrollbar(
            thumbVisibility: true,
            child: ListView(
              padding: const EdgeInsets.only(top: 24.0),
              children: [
                const SizedBox(height: 8.0),
                SizedCardFront(
                  cardFront: card.front,
                  displayWidth: MediaQuery.of(context).size.width - (_kCardDisplayPaddingHorizontal * 2),
                ),
                const SizedBox(height: 32.0),
                const Divider(height: 1),
                InfoRow(
                  icon: Icons.description_outlined,
                  title: locale.cardSummaryScreenCardDataCta,
                  subtitle: locale.cardSummaryScreenCardDataIssuedBy(summary.issuer.shortName),
                  onTap: () => _onCardDataPressed(context, card),
                ),
                const Divider(height: 1),
                InfoRow(
                  icon: Icons.history_outlined,
                  title: locale.cardSummaryScreenCardHistoryCta,
                  subtitle: _createInteractionText(context, summary.latestSuccessInteraction),
                  onTap: () => _onCardHistoryPressed(context, card.id),
                ),
                const Divider(height: 1),
                InfoRow(
                  icon: Icons.replay_outlined,
                  title: locale.cardSummaryScreenCardUpdateCta,
                  subtitle: _createOperationText(context, summary.latestIssuedOperation),
                  onTap: () => _onCardUpdatePressed(context, card),
                ),
                const Divider(height: 1),
                if (card.config.removable) ...[
                  InfoRow(
                    icon: Icons.delete_outline_rounded,
                    title: locale.cardSummaryScreenCardDeleteCta,
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
    final locale = AppLocalizations.of(context);
    ExplanationSheet.show(
      context,
      title: locale.cardSummaryScreenNoUpdateAvailableSheetTitle,
      description: locale.cardSummaryScreenNoUpdateAvailableSheetDescription,
      closeButtonText: locale.cardSummaryScreenNoUpdateAvailableSheetCloseCta,
    );
  }

  String _createInteractionText(BuildContext context, InteractionTimelineAttribute? attribute) {
    final locale = AppLocalizations.of(context);
    if (attribute != null) {
      final String timeAgo = TimeAgoFormatter.format(locale, attribute.dateTime);
      final String status = TimelineAttributeStatusTextMapper.map(locale, attribute).toLowerCase();
      return locale.cardSummaryScreenLatestSuccessInteraction(timeAgo, status, attribute.organization.shortName);
    } else {
      return locale.cardSummaryScreenLatestSuccessInteractionUnknown;
    }
  }

  String _createOperationText(BuildContext context, OperationTimelineAttribute? attribute) {
    final locale = AppLocalizations.of(context);
    if (attribute != null) {
      DateTime issued = attribute.dateTime;
      String issuedTime = OperationIssuedTimeFormatter.format(locale, issued);
      String issuedText = locale.cardSummaryScreenLatestIssuedOperation(issuedTime);

      DateTime validUntil = issued.add(const Duration(days: _kCardExpiresInDays));
      String validUntilTime = CardValidUntilTimeFormatter.format(locale, validUntil);
      String validUntilText = locale.cardSummaryScreenCardValidUntil(validUntilTime);

      return '$issuedText\n$validUntilText';
    } else {
      return locale.cardSummaryScreenLatestIssuedOperationUnknown;
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
            child: Text(AppLocalizations.of(context).generalRetry),
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
