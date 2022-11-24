import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_sticky_header/flutter_sticky_header.dart';

import '../../../domain/model/card_front.dart';
import '../../../domain/model/timeline_attribute.dart';
import '../../../util/extension/date_time_extension.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/text_icon_button.dart';
import '../../common/widget/wallet_card_front.dart';
import 'bloc/card_history_bloc.dart';
import 'widget/timeline_header.dart';
import 'widget/timeline_row.dart';

const double _kCardFrontScaleFactor = 0.35;

class CardHistoryScreen extends StatelessWidget {
  static String getArguments(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args as String;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in a (mock) id when opening the CardSummaryScreen');
    }
  }

  const CardHistoryScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(AppLocalizations.of(context).cardHistoryScreenTitle),
      ),
      body: _buildBody(context),
    );
  }

  Widget _buildBody(BuildContext context) {
    return BlocBuilder<CardHistoryBloc, CardHistoryState>(
      builder: (context, state) {
        if (state is CardHistoryInitial) return _buildLoading();
        if (state is CardHistoryLoadInProgress) return _buildLoading();
        if (state is CardHistoryLoadSuccess) return _buildHistory(context, state);
        throw UnsupportedError('Unknown state: $state');
      },
    );
  }

  Widget _buildLoading() {
    return const CenteredLoadingIndicator();
  }

  Widget _buildHeader(BuildContext context, CardFront cardFront) {
    const walletCardOriginalHeight = kWalletCardHeight;
    const walletCardScaledWidth = kWalletCardWidth * _kCardFrontScaleFactor;
    const walletCardScaledHeight = kWalletCardHeight * _kCardFrontScaleFactor;

    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
          child: Column(
            children: [
              SizedBox(
                width: walletCardScaledWidth,
                height: walletCardScaledHeight,
                child: FittedBox(
                  alignment: Alignment.center,
                  child: SizedBox(
                    height: walletCardOriginalHeight,
                    child: WalletCardFront(cardFront: cardFront, onPressed: null),
                  ),
                ),
              ),
              const SizedBox(height: 24),
              Text(
                cardFront.title,
                style: Theme.of(context).textTheme.headline2,
              ),
            ],
          ),
        ),
        const Divider(height: 1),
      ],
    );
  }

  Widget _buildHistory(BuildContext context, CardHistoryLoadSuccess state) {
    final List<Widget> slivers = [];

    // Card front & title
    slivers.add(
      SliverToBoxAdapter(
        child: _buildHeader(context, state.card.front),
      ),
    );

    // Timeline
    final Map<DateTime, List<TimelineAttribute>> monthYearMap = _monthYearAttributeMap(state.attributes);
    monthYearMap.forEach((dateTime, values) {
      slivers.add(
        SliverStickyHeader(
          header: TimelineHeader(dateTime: dateTime),
          sliver: SliverList(
            delegate: SliverChildBuilderDelegate(
              (context, i) => TimelineRow(attribute: values[i]),
              childCount: values.length,
            ),
          ),
        ),
      );
    });

    // Close button
    slivers.add(
      SliverFillRemaining(
        hasScrollBody: false,
        fillOverscroll: true,
        child: _buildBackButton(context),
      ),
    );

    return CustomScrollView(slivers: slivers);
  }

  Widget _buildBackButton(BuildContext context) {
    return Align(
      alignment: Alignment.bottomCenter,
      child: SizedBox(
        height: 72,
        width: double.infinity,
        child: TextIconButton(
          onPressed: () => Navigator.pop(context),
          arrowPosition: ArrowPosition.start,
          icon: Icons.arrow_back,
          child: Text(AppLocalizations.of(context).cardHistoryScreenBackCta),
        ),
      ),
    );
  }

  Map<DateTime, List<TimelineAttribute>> _monthYearAttributeMap(List<TimelineAttribute> attributes) {
    Map<DateTime, List<TimelineAttribute>> map = {};

    for (TimelineAttribute attribute in attributes) {
      final DateTime yearMonthKey = attribute.dateTime.yearMonthOnly();

      List<TimelineAttribute>? mapEntry = map[yearMonthKey];
      if (mapEntry != null) {
        mapEntry.add(attribute);
      } else {
        map[yearMonthKey] = [attribute];
      }
    }
    return map;
  }
}
