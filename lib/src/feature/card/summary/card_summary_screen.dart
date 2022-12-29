import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/data_highlight.dart';
import '../../../domain/model/timeline_attribute.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../domain/model/wallet_card_summary.dart';
import '../../../domain/usecase/card/get_wallet_card_update_issuance_request_id_usecase.dart';
import '../../../util/formatter/time_ago_formatter.dart';
import '../../../util/mapper/timeline_attribute_type_mapper.dart';
import '../../../wallet_routes.dart';
import '../../common/widget/attribute/data_attribute_row_image.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/explanation_sheet.dart';
import '../../common/widget/link_button.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../common/widget/text_icon_button.dart';
import '../../common/widget/wallet_card_front.dart';
import '../../issuance/argument/issuance_screen_argument.dart';
import 'bloc/card_summary_bloc.dart';

class CardSummaryScreen extends StatelessWidget {
  static String getArguments(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args as String;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in a (mock) id when opening the CardSummaryScreen');
    }
  }

  const CardSummaryScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: _buildAppBar(context),
      body: _buildBody(context),
      floatingActionButton: _buildFAB(context),
      floatingActionButtonLocation: FloatingActionButtonLocation.centerFloat,
    );
  }

  PreferredSizeWidget _buildAppBar(BuildContext context) {
    return AppBar(title: Text(AppLocalizations.of(context).cardSummaryScreenTitle));
  }

  Widget _buildFAB(BuildContext context) {
    return BlocBuilder<CardSummaryBloc, CardSummaryState>(
      builder: (context, state) {
        if (state is CardSummaryLoadSuccess) {
          return FloatingActionButton.extended(
            onPressed: () => _onCardDataSharePressed(context, state.summary.card.front.title),
            label: Text(AppLocalizations.of(context).cardSummaryScreenShareCta),
            icon: const Icon(Icons.qr_code),
          );
        } else {
          return const SizedBox.shrink();
        }
      },
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

  Widget _buildLoading() {
    return const CenteredLoadingIndicator();
  }

  Widget _buildSummary(BuildContext context, WalletCardSummary summary) {
    final locale = AppLocalizations.of(context);

    return Scrollbar(
      child: ListView(
        padding: const EdgeInsets.symmetric(vertical: 24.0),
        children: [
          _buildCardFront(summary.card),
          const SizedBox(height: 24.0),
          const Divider(),
          const SizedBox(height: 24.0),
          _buildDataHighlight(context, summary.card.id, summary.dataHighlight),
          const SizedBox(height: 8.0),
          const Divider(),
          const SizedBox(height: 24.0),
          _buildInteractionHighlight(context, summary.card.id, summary.interactionAttribute),
          const SizedBox(height: 8.0),
          const Divider(),
          Align(
            alignment: AlignmentDirectional.centerStart,
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 24.0),
              child: TextIconButton(
                icon: Icons.replay,
                iconPosition: IconPosition.start,
                onPressed: () => _onRefreshPressed(context, summary.card),
                child: Text(locale.cardSummaryScreenCardRenewCta),
              ),
            ),
          ),
          const Divider(),
          Align(
            alignment: AlignmentDirectional.centerStart,
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 24.0),
              child: TextIconButton(
                icon: Icons.delete,
                iconPosition: IconPosition.start,
                onPressed: () => PlaceholderScreen.show(context),
                child: Text(locale.cardSummaryScreenCardDeleteCta),
              ),
            ),
          ),
          const Divider(),
          const SizedBox(height: kFloatingActionButtonMargin + 64),
        ],
      ),
    );
  }

  /// Temporary async logic inside [CardSummaryScreen] class;
  /// This async flow isn't designed (happy & unhappy paths); it's to do for after demo day.
  void _onRefreshPressed(BuildContext context, WalletCard card) {
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

  void _showNoUpdateAvailableSheet(BuildContext context) {
    final locale = AppLocalizations.of(context);
    ExplanationSheet.show(
      context,
      title: locale.cardSummaryScreenNoUpdateAvailableSheetTitle,
      description: locale.cardSummaryScreenNoUpdateAvailableSheetDescription,
      closeButtonText: locale.cardSummaryScreenNoUpdateAvailableSheetCloseCta,
    );
  }

  Widget _buildCardFront(WalletCard walletCard) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16.0),
      child: WalletCardFront(cardFront: walletCard.front, onPressed: null),
    );
  }

  Widget _buildDataHighlight(BuildContext context, String cardId, DataHighlight highlight) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 24.0),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        mainAxisSize: MainAxisSize.max,
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 8.0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        AppLocalizations.of(context).cardSummaryScreenDataAttributesTitle,
                        style: Theme.of(context).textTheme.subtitle1,
                      ),
                      const SizedBox(height: 8.0),
                      Text(
                        highlight.title,
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                      ),
                      Visibility(
                        visible: highlight.subtitle?.isNotEmpty ?? false,
                        child: Text(
                          highlight.subtitle ?? '',
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                    ],
                  ),
                ),
                LinkButton(
                  onPressed: () => _onCardDataPressed(context, cardId),
                  child: Text(AppLocalizations.of(context).cardSummaryScreenAddCardCta),
                ),
              ],
            ),
          ),
          if (highlight.image != null) ...[
            const SizedBox(width: 16.0),
            DataAttributeRowImage(image: AssetImage(highlight.image!)),
          ],
        ],
      ),
    );
  }

  Widget _buildInteractionHighlight(BuildContext context, String cardId, InteractionAttribute? attribute) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 24.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 8.0),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  AppLocalizations.of(context).cardSummaryScreenShareHistoryTitle,
                  style: Theme.of(context).textTheme.subtitle1,
                ),
                const SizedBox(height: 8.0),
                Text(_createInteractionText(context, attribute), maxLines: 2, overflow: TextOverflow.ellipsis),
              ],
            ),
          ),
          LinkButton(
            onPressed: () => _onCardHistoryPressed(context, cardId),
            child: Text(AppLocalizations.of(context).cardSummaryScreenShareHistoryAllCta),
          ),
        ],
      ),
    );
  }

  String _createInteractionText(BuildContext context, InteractionAttribute? attribute) {
    final locale = AppLocalizations.of(context);
    if (attribute != null) {
      final String timeAgo = TimeAgoFormatter.format(locale, attribute.dateTime);
      final String status = TimelineAttributeTypeTextMapper.map(locale, attribute).toLowerCase();
      return locale.cardSummaryScreenShareHistory(timeAgo, status, attribute.organization.shortName);
    } else {
      return locale.cardSummaryScreenShareSuccessNoHistory;
    }
  }

  void _onCardDataPressed(BuildContext context, String cardId) {
    Navigator.restorablePushNamed(context, WalletRoutes.cardDataRoute, arguments: cardId);
  }

  void _onCardHistoryPressed(BuildContext context, String cardId) {
    Navigator.restorablePushNamed(context, WalletRoutes.cardHistoryRoute, arguments: cardId);
  }

  void _onCardDataSharePressed(BuildContext context, String screenTitle) {
    Navigator.restorablePushNamed(context, WalletRoutes.cardShareRoute, arguments: screenTitle);
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
}
