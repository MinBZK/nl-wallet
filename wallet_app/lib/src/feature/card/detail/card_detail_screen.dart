import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../domain/model/wallet_card_detail.dart';
import '../../../domain/usecase/card/get_wallet_card_update_issuance_request_id_usecase.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/formatter/card_valid_until_time_formatter.dart';
import '../../../util/formatter/operation_issued_time_formatter.dart';
import '../../../util/formatter/time_ago_formatter.dart';
import '../../../util/formatter/timeline_attribute_status_formatter.dart';
import '../../../wallet_feature_flags.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/sheet/explanation_sheet.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/card/wallet_card_item.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/info_row.dart';
import '../../issuance/argument/issuance_screen_argument.dart';
import '../data/argument/card_data_screen_argument.dart';
import 'argument/card_detail_screen_argument.dart';
import 'bloc/card_detail_bloc.dart';

const _kCardExpiresInDays = 365; // 1 year for demo purposes

class CardDetailScreen extends StatelessWidget {
  static CardDetailScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return CardDetailScreenArgument.fromMap(args as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in [CardDetailScreenArgument] when opening the CardDetailScreen');
    }
  }

  final String cardTitle;

  const CardDetailScreen({required this.cardTitle, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: _buildAppBar(context),
      body: SafeArea(
        child: _buildBody(context),
      ),
    );
  }

  PreferredSizeWidget _buildAppBar(BuildContext context) {
    final fallbackAppBarTitleText = Text(cardTitle);
    return AppBar(
      title: BlocBuilder<CardDetailBloc, CardDetailState>(
        builder: (context, state) {
          return switch (state) {
            CardDetailInitial() => fallbackAppBarTitleText,
            CardDetailLoadInProgress() => fallbackAppBarTitleText,
            CardDetailLoadSuccess() => Text(state.detail.card.front.title),
            CardDetailLoadFailure() => fallbackAppBarTitleText,
          };
        },
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    return BlocBuilder<CardDetailBloc, CardDetailState>(
      builder: (context, state) {
        return switch (state) {
          CardDetailInitial() => _buildLoading(),
          CardDetailLoadInProgress() => _buildLoading(),
          CardDetailLoadSuccess() => _buildDetail(context, state.detail),
          CardDetailLoadFailure() => _buildError(context, state),
        };
      },
    );
  }

  Widget _buildLoading() => const CenteredLoadingIndicator();

  Widget _buildDetail(BuildContext context, WalletCardDetail detail) {
    final card = detail.card;

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
                  title: Text(context.l10n.cardDetailScreenCardDataCta),
                  subtitle: Text(context.l10n.cardDetailScreenCardDataIssuedBy(detail.issuer.shortName)),
                  onTap: () => _onCardDataPressed(context, card),
                ),
                const Divider(height: 1),
                InfoRow(
                  icon: Icons.history_outlined,
                  title: Text(context.l10n.cardDetailScreenCardHistoryCta),
                  subtitle: Text(_createInteractionText(context, detail.latestSuccessInteraction)),
                  onTap: () => _onCardHistoryPressed(context, card.id),
                ),
                const Divider(height: 1),
                if (card.config.updatable) ...[
                  InfoRow(
                    icon: Icons.replay_outlined,
                    title: Text(context.l10n.cardDetailScreenCardUpdateCta),
                    subtitle: Text(_createOperationText(context, detail.latestIssuedOperation)),
                    onTap: () => _onCardUpdatePressed(context, card),
                  ),
                  const Divider(height: 1),
                ],
                if (card.config.removable) ...[
                  InfoRow(
                    icon: Icons.delete_outline_rounded,
                    title: Text(context.l10n.cardDetailScreenCardDeleteCta),
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
      title: context.l10n.cardDetailScreenNoUpdateAvailableSheetTitle,
      description: context.l10n.cardDetailScreenNoUpdateAvailableSheetDescription,
      closeButtonText: context.l10n.cardDetailScreenNoUpdateAvailableSheetCloseCta,
    );
  }

  String _createInteractionText(BuildContext context, InteractionTimelineAttribute? attribute) {
    if (attribute != null) {
      final String timeAgo = TimeAgoFormatter.format(context, attribute.dateTime);
      final String status = TimelineAttributeStatusTextFormatter.map(context, attribute).toLowerCase();
      return context.l10n.cardDetailScreenLatestSuccessInteraction(
        attribute.organization.shortName,
        status,
        timeAgo,
      );
    } else {
      return context.l10n.cardDetailScreenLatestSuccessInteractionUnknown;
    }
  }

  String _createOperationText(BuildContext context, OperationTimelineAttribute? attribute) {
    if (attribute != null) {
      DateTime issued = attribute.dateTime;
      String issuedTime = OperationIssuedTimeFormatter.format(context, issued);
      String issuedText = context.l10n.cardDetailScreenLatestIssuedOperation(issuedTime);

      DateTime validUntil = issued.add(const Duration(days: _kCardExpiresInDays));
      String validUntilTime = CardValidUntilTimeFormatter.format(context, validUntil);
      String validUntilText = context.l10n.cardDetailScreenCardValidUntil(validUntilTime);

      return '$issuedText\n$validUntilText';
    } else {
      return context.l10n.cardDetailScreenLatestIssuedOperationUnknown;
    }
  }

  Widget _buildError(BuildContext context, CardDetailLoadFailure state) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.error_outline),
          const SizedBox(height: 16),
          TextButton(
            child: Text(context.l10n.generalRetry),
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
        cardTitle: card.front.title,
      ).toMap(),
    );
  }

  void _onCardHistoryPressed(BuildContext context, String cardId) {
    if (WalletFeatureFlags.enableCardHistoryOverview) {
      Navigator.restorablePushNamed(context, WalletRoutes.cardHistoryRoute, arguments: cardId);
    } else {
      PlaceholderScreen.show(context);
    }
  }

  /// Temporary async logic inside [CardDetailScreen] class;
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
