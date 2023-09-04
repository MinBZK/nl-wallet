import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/timeline/timeline_section.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/timeline/timeline_section_list_factory.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/history/timeline_section_sliver.dart';
import '../detail/argument/history_detail_screen_argument.dart';
import 'bloc/history_overview_bloc.dart';

class HistoryOverviewScreen extends StatelessWidget {
  const HistoryOverviewScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(context.l10n.historyOverviewScreenTitle),
      ),
      body: SafeArea(
        child: _buildBody(context),
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    return BlocBuilder<HistoryOverviewBloc, HistoryOverviewState>(
      builder: (context, state) {
        return switch (state) {
          HistoryOverviewInitial() => _buildLoading(),
          HistoryOverviewLoadInProgress() => _buildLoading(),
          HistoryOverviewLoadSuccess() => _buildTimeline(context, state),
          HistoryOverviewLoadFailure() => _buildError(context),
        };
      },
    );
  }

  Widget _buildLoading() {
    return const CenteredLoadingIndicator();
  }

  Widget _buildTimeline(BuildContext context, HistoryOverviewLoadSuccess state) {
    final List<TimelineSection> sections = TimelineSectionListFactory.create(state.attributes);

    List<Widget> slivers = [
      ...sections.map(
        (section) => TimelineSectionSliver(
          section: section,
          onRowPressed: (timelineAttributeId) => _onTimelineRowPressed(context, timelineAttributeId),
        ),
      ),
    ];

    return Column(
      children: [
        Expanded(
          child: Scrollbar(
            child: CustomScrollView(
              slivers: slivers,
            ),
          ),
        ),
        Container(
          color: context.colorScheme.background,
          child: const BottomBackButton(showDivider: true),
        ),
      ],
    );
  }

  void _onTimelineRowPressed(BuildContext context, String timelineAttributeId) {
    Navigator.restorablePushNamed(
      context,
      WalletRoutes.historyDetailRoute,
      arguments: HistoryDetailScreenArgument(
        timelineAttributeId: timelineAttributeId,
      ).toMap(),
    );
  }

  Widget _buildError(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          const Spacer(),
          Text(
            context.l10n.errorScreenGenericDescription,
            textAlign: TextAlign.center,
          ),
          const Spacer(),
          ElevatedButton(
            onPressed: () {
              context.read<HistoryOverviewBloc>().add(const HistoryOverviewLoadTriggered());
            },
            child: Text(context.l10n.generalRetry),
          ),
        ],
      ),
    );
  }
}
