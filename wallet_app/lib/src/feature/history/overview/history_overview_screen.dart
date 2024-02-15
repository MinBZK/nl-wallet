import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/timeline/timeline_attribute.dart';
import '../../../domain/model/timeline/timeline_section.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/timeline/timeline_section_list_factory.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/history/timeline_section_sliver.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';
import '../detail/argument/history_detail_screen_argument.dart';
import 'bloc/history_overview_bloc.dart';

class HistoryOverviewScreen extends StatelessWidget {
  const HistoryOverviewScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      key: const Key('historyOverviewScreen'),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(child: _buildContent(context)),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    return BlocBuilder<HistoryOverviewBloc, HistoryOverviewState>(
      builder: (context, state) {
        final slivers = switch (state) {
          HistoryOverviewInitial() => [_buildLoadingSliver()],
          HistoryOverviewLoadInProgress() => [_buildLoadingSliver()],
          HistoryOverviewLoadSuccess() => _buildTimelineSliver(context, state),
          HistoryOverviewLoadFailure() => [_buildErrorSliver(context)],
        };
        return Scrollbar(
          child: CustomScrollView(
            slivers: [
              SliverWalletAppBar(title: context.l10n.historyOverviewScreenTitle),
              ...slivers,
            ],
          ),
        );
      },
    );
  }

  Widget _buildLoadingSliver() {
    return const SliverFillRemaining(
      child: CenteredLoadingIndicator(),
    );
  }

  List<Widget> _buildTimelineSliver(BuildContext context, HistoryOverviewLoadSuccess state) {
    final List<TimelineSection> sections = TimelineSectionListFactory.create(state.attributes);
    final List<Widget> slivers = sections
        .map(
          (section) => TimelineSectionSliver(
            section: section,
            onRowPressed: (attribute) => _onTimelineRowPressed(context, attribute),
          ),
        )
        .toList();

    return [...slivers, const SliverSizedBox(height: 24)];
  }

  void _onTimelineRowPressed(BuildContext context, TimelineAttribute attribute) {
    Navigator.pushNamed(
      context,
      WalletRoutes.historyDetailRoute,
      arguments: HistoryDetailScreenArgument(
        timelineAttribute: attribute,
      ).toMap(),
    );
  }

  Widget _buildErrorSliver(BuildContext context) {
    return SliverFillRemaining(
      hasScrollBody: false,
      child: Padding(
        padding: const EdgeInsets.all(16),
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
      ),
    );
  }
}
