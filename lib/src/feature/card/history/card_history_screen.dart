import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/card_front.dart';
import '../../../domain/model/timeline/timeline_section.dart';
import '../../../util/timeline/timeline_section_list_factory.dart';
import '../../../wallet_routes.dart';
import '../../common/widget/bottom_back_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/history/timeline_card_header.dart';
import '../../common/widget/history/timeline_section_sliver.dart';
import 'bloc/card_history_bloc.dart';

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

  Widget _buildHistory(BuildContext context, CardHistoryLoadSuccess state) {
    final List<TimelineSection> sections = TimelineSectionListFactory.create(state.attributes);

    List<Widget> slivers = [
      _buildCardHeader(state.card.front),
      ...sections.map((section) => TimelineSectionSliver(
            section: section,
            onRowPressed: (timelineAttributeId) => _onTimelineRowPressed(context, timelineAttributeId),
          )),
    ];

    return Column(
      children: [
        Expanded(
          child: Scrollbar(
            thumbVisibility: true,
            child: CustomScrollView(
              slivers: slivers,
            ),
          ),
        ),
        const BottomBackButton(),
      ],
    );
  }

  Widget _buildCardHeader(CardFront front) {
    return SliverToBoxAdapter(child: TimelineCardHeader(cardFront: front));
  }

  void _onTimelineRowPressed(BuildContext context, String timelineAttributeId) {
    Navigator.restorablePushNamed(context, WalletRoutes.historyDetailRoute, arguments: timelineAttributeId);
  }
}
