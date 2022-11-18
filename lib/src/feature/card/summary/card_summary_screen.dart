import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/data_highlight.dart';
import '../../../domain/model/timeline_attribute.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../domain/model/wallet_card_summary.dart';
import '../../../util/formatter/time_ago_formatter.dart';
import '../../../util/mapper/timeline_attribute_type_text_mapper.dart';
import '../../../wallet_routes.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/data_attribute_image.dart';
import '../../common/widget/link_button.dart';
import '../../common/widget/wallet_card_front.dart';
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
            onPressed: () => _onCardDataSharePressed(context),
            label: Text(AppLocalizations.of(context).cardSummaryDataShareCta),
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
      buildWhen: (previous, current) => current is CardSummaryLoadSuccess,
      builder: (context, state) {
        if (state is CardSummaryInitial) return _buildLoading();
        if (state is CardSummaryLoadInProgress) return _buildLoading();
        if (state is CardSummaryLoadSuccess) return _buildSummary(context, state.summary);
        throw UnsupportedError('Unknown state: $state');
      },
    );
  }

  Widget _buildLoading() {
    return const CenteredLoadingIndicator();
  }

  Widget _buildSummary(BuildContext context, WalletCardSummary summary) {
    return ListView(
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
        const SizedBox(height: 8.0),
        Align(
          alignment: AlignmentDirectional.centerStart,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 24.0),
            child: LinkButton(
              child: Text(AppLocalizations.of(context).cardSummaryOptionsCta),
              onPressed: () => _onCardOptionsPressed(context),
            ),
          ),
        ),
        const SizedBox(height: 8.0),
        const Divider(),
        const SizedBox(height: kFloatingActionButtonMargin + 64),
      ],
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
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 8.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      AppLocalizations.of(context).cardSummaryDataAttributesTitle,
                      style: Theme.of(context).textTheme.subtitle1,
                    ),
                    const SizedBox(height: 8.0),
                    Text(highlight.title, maxLines: 1, overflow: TextOverflow.ellipsis),
                    Visibility(
                      visible: highlight.subtitle?.isNotEmpty ?? false,
                      child: Text(highlight.subtitle ?? '', maxLines: 1, overflow: TextOverflow.ellipsis),
                    ),
                  ],
                ),
              ),
              LinkButton(
                onPressed: () => _onCardDataPressed(context, cardId),
                child: Text(AppLocalizations.of(context).cardSummaryDataAttributesAllCta),
              ),
            ],
          ),
          if (highlight.image != null) ...[
            const SizedBox(width: 16.0),
            DataAttributeImage(image: AssetImage(highlight.image!)),
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
                  AppLocalizations.of(context).cardSummaryDataShareHistoryTitle,
                  style: Theme.of(context).textTheme.subtitle1,
                ),
                const SizedBox(height: 8.0),
                Text(_createInteractionText(context, attribute), maxLines: 2, overflow: TextOverflow.ellipsis),
              ],
            ),
          ),
          LinkButton(
            onPressed: () => _onCardHistoryPressed(context, cardId),
            child: Text(AppLocalizations.of(context).cardSummaryDataShareHistoryAllCta),
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
      return locale.cardSummaryDataShareHistory(timeAgo, status, attribute.organization);
    } else {
      return locale.cardSummaryDataShareSuccessNoHistory;
    }
  }

  void _onCardOptionsPressed(BuildContext context) {
    Fimber.d('_onCardOptionsPressed');
  }

  void _onCardDataPressed(BuildContext context, String cardId) {
    Navigator.restorablePushNamed(context, WalletRoutes.cardDataRoute, arguments: cardId);
  }

  void _onCardHistoryPressed(BuildContext context, String cardId) {
    Navigator.restorablePushNamed(context, WalletRoutes.cardHistoryRoute, arguments: cardId);
  }

  void _onCardDataSharePressed(BuildContext context) {
    Fimber.d('_onCardDataSharePressed');
  }
}
