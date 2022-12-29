import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/timeline_section.dart';
import '../../../util/timeline/timeline_section_list_factory.dart';
import '../../../wallet_routes.dart';
import '../../common/widget/bottom_back_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/history/timeline_section_sliver.dart';
import 'bloc/history_overview_bloc.dart';

class HistoryOverviewScreen extends StatelessWidget {
  const HistoryOverviewScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(AppLocalizations.of(context).historyOverviewScreenTitle),
      ),
      body: _buildBody(context),
    );
  }

  Widget _buildBody(BuildContext context) {
    return BlocBuilder<HistoryOverviewBloc, HistoryOverviewState>(
      builder: (context, state) {
        if (state is HistoryOverviewInitial) return _buildLoading();
        if (state is HistoryOverviewLoadInProgress) return _buildLoading();
        if (state is HistoryOverviewLoadSuccess) return _buildTimeline(context, state);
        throw UnsupportedError('Unknown state: $state');
      },
    );
  }

  Widget _buildLoading() {
    return const CenteredLoadingIndicator();
  }

  Widget _buildTimeline(BuildContext context, HistoryOverviewLoadSuccess state) {
    final List<TimelineSection> sections = TimelineSectionListFactory.create(state.attributes);

    List<Widget> slivers = [
      ...sections.map((section) => TimelineSectionSliver(
            section: section,
            onRowPressed: (timelineAttributeId) => _onTimelineRowPressed(context, timelineAttributeId),
          )),
      _buildBackButton(context),
    ];

    return Scrollbar(
      child: CustomScrollView(
        slivers: slivers,
      ),
    );
  }

  Widget _buildBackButton(BuildContext context) {
    return const SliverFillRemaining(
      hasScrollBody: false,
      fillOverscroll: true,
      child: BottomBackButton(),
    );
  }

  void _onTimelineRowPressed(BuildContext context, String timelineAttributeId) {
    Navigator.restorablePushNamed(context, WalletRoutes.historyDetailRoute, arguments: timelineAttributeId);
  }
}
