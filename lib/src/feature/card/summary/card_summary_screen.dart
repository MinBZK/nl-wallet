import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/wallet_card.dart';
import '../../../domain/model/wallet_card_data_attribute.dart';
import '../../../domain/model/wallet_card_summary.dart';
import '../../../wallet_routes.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/link_button.dart';
import '../../common/widget/wallet_card_front.dart';
import 'bloc/card_summary_bloc.dart';

const _kHighlightImageBorderRadius = 4.0;

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
            label: Text(AppLocalizations.of(context).cardSummaryDataShareButton),
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
        _buildDataHighlight(context, summary.data),
        const SizedBox(height: 8.0),
        const Divider(),
        const SizedBox(height: 24.0),
        _buildUsageHighlight(context, summary.usage),
        const SizedBox(height: 8.0),
        const Divider(),
        const SizedBox(height: 8.0),
        Align(
          alignment: AlignmentDirectional.centerStart,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 24.0),
            child: LinkButton(
              child: Text(AppLocalizations.of(context).cardSummaryOptionsButton),
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
      child: WalletCardFront(walletCard: walletCard, onPressed: null),
    );
  }

  Widget _buildDataHighlight(BuildContext context, WalletCardDataAttribute dataAttribute) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 24.0),
      child: Row(
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
                        AppLocalizations.of(context).cardSummaryDataAttributesTitle,
                        style: Theme.of(context).textTheme.subtitle1,
                      ),
                      const SizedBox(height: 8.0),
                      Text(dataAttribute.content, maxLines: 2, overflow: TextOverflow.ellipsis),
                    ],
                  ),
                ),
                LinkButton(
                  onPressed: () => _onCardDataPressed(context),
                  child: Text(AppLocalizations.of(context).cardSummaryDataAttributesAllButton),
                ),
              ],
            ),
          ),
          if (dataAttribute.image != null) ...[
            const SizedBox(width: 16.0),
            ClipRRect(
              borderRadius: const BorderRadius.all(Radius.circular(_kHighlightImageBorderRadius)),
              child: Image.asset(dataAttribute.image ?? ''),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildUsageHighlight(BuildContext context, WalletCardDataAttribute dataAttribute) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 24.0),
      child: Row(
        mainAxisSize: MainAxisSize.max,
        mainAxisAlignment: MainAxisAlignment.start,
        children: [
          Expanded(
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
                      Text(dataAttribute.content, maxLines: 2, overflow: TextOverflow.ellipsis),
                    ],
                  ),
                ),
                LinkButton(
                  onPressed: () => _onCardHistoryPressed(context),
                  child: Text(AppLocalizations.of(context).cardSummaryDataShareHistoryAllButton),
                ),
              ],
            ),
          ),
          if (dataAttribute.image != null) ...[
            const SizedBox(width: 16.0),
            ClipRRect(
              borderRadius: const BorderRadius.all(Radius.circular(_kHighlightImageBorderRadius)),
              child: Image.asset(dataAttribute.image ?? ''),
            ),
          ],
        ],
      ),
    );
  }

  void _onCardOptionsPressed(BuildContext context) {
    Fimber.d('_onCardOptionsPressed');
  }

  void _onCardDataPressed(BuildContext context) {
    Navigator.restorablePushNamed(context, WalletRoutes.cardDataRoute);
  }

  void _onCardHistoryPressed(BuildContext context) {
    Navigator.restorablePushNamed(context, WalletRoutes.cardHistoryRoute);
  }

  void _onCardDataSharePressed(BuildContext context) {
    Fimber.d('_onCardDataSharePressed');
  }
}
