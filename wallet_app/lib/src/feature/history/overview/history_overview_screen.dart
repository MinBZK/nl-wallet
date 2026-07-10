import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/event/event_section.dart';
import '../../../domain/model/event/wallet_event.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/cast_util.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_constants.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/history/history_section_sliver.dart';
import '../../common/widget/loading_indicator.dart';
import '../../common/widget/spacer/sliver_sized_box.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_app_bar.dart';
import '../../common/widget/wallet_scrollbar.dart';
import '../detail/argument/history_detail_screen_argument.dart';
import 'bloc/history_overview_bloc.dart';

/// Distance from the bottom (in pixels) at which the next page is pre-fetched.
const double _kScrollLoadMoreThreshold = 400;

class HistoryOverviewScreen extends StatefulWidget {
  const HistoryOverviewScreen({super.key});

  @override
  State<HistoryOverviewScreen> createState() => _HistoryOverviewScreenState();
}

class _HistoryOverviewScreenState extends State<HistoryOverviewScreen> {
  final _scrollController = ScrollController();

  /// Whether there are more pages available to be loaded when scrolling.
  bool _hasNextPage = false;

  @override
  void initState() {
    super.initState();
    _scrollController.addListener(_onScroll);
  }

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  void _onScroll() {
    // Avoid adding events when there are no more pages to load.
    if (!_hasNextPage) return;

    final position = _scrollController.position;
    if (position.extentAfter <= _kScrollLoadMoreThreshold) {
      context.read<HistoryOverviewBloc>().add(const HistoryOverviewLoadNextPageTriggered());
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.historyOverviewScreenTitle),
      ),
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
    return BlocConsumer<HistoryOverviewBloc, HistoryOverviewState>(
      listener: (_, state) => _hasNextPage = tryCast<HistoryOverviewLoadSuccess>(state)?.hasNextPage ?? false,
      builder: (context, state) {
        final content = switch (state) {
          HistoryOverviewInitial() => _buildLoadingSliver(context),
          HistoryOverviewLoadInProgress() => _buildLoadingSliver(context),
          HistoryOverviewLoadSuccess() => _buildSectionedEventsSliver(context, state),
          HistoryOverviewLoadFailure() => _buildErrorSliver(context),
        };
        return WalletScrollbar(
          controller: _scrollController,
          child: CustomScrollView(
            controller: _scrollController,
            slivers: [
              content,
            ],
          ),
        );
      },
    );
  }

  Widget _buildLoadingSliver(BuildContext context) {
    return SliverMainAxisGroup(
      slivers: [
        SliverToBoxAdapter(
          child: Padding(
            padding: kDefaultTitlePadding,
            child: TitleText(context.l10n.historyOverviewScreenTitle),
          ),
        ),
        const SliverFillRemaining(
          hasScrollBody: false,
          child: CenteredLoadingIndicator(),
        ),
      ],
    );
  }

  Widget _buildSectionedEventsSliver(BuildContext context, HistoryOverviewLoadSuccess state) {
    final List<Widget> slivers = state.eventsByYearMonth.entries
        .map(
          (entry) => HistorySectionSliver(
            section: EventSection(entry.key, entry.value),
            onRowPressed: (event) => _onEventPressed(context, event),
          ),
        )
        .toList();

    return SliverMainAxisGroup(
      slivers: [
        SliverToBoxAdapter(
          child: Padding(
            padding: const EdgeInsets.only(left: 16, right: 16, top: 12, bottom: 24),
            child: TitleText(context.l10n.historyOverviewScreenTitle),
          ),
        ),
        ...slivers,
        if (state.isLoadingMore) _buildLoadingNextPageSliver() else const SliverSizedBox(height: 24),
      ],
    );
  }

  Widget _buildLoadingNextPageSliver() {
    return SliverToBoxAdapter(
      child: Container(
        alignment: .center,
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
        child: Row(
          mainAxisSize: .min,
          mainAxisAlignment: .center,
          crossAxisAlignment: .center,
          children: [
            const SizedBox(
              height: 24,
              width: 24,
              child: LoadingIndicator(),
            ),
            const SizedBox(width: 16),
            Text.rich(
              context.l10n.historyOverviewScreenLoadingNextPage.toTextSpan(context),
              style: context.textTheme.bodyLarge,
            ),
          ],
        ),
      ),
    );
  }

  void _onEventPressed(BuildContext context, WalletEvent event) {
    Navigator.pushNamed(
      context,
      WalletRoutes.historyDetailRoute,
      arguments: HistoryDetailScreenArgument(walletEvent: event).toMap(),
    );
  }

  Widget _buildErrorSliver(BuildContext context) {
    return SliverMainAxisGroup(
      slivers: [
        SliverToBoxAdapter(
          child: Padding(
            padding: kDefaultTitlePadding,
            child: TitleText(context.l10n.historyOverviewScreenTitle),
          ),
        ),
        SliverFillRemaining(
          hasScrollBody: false,
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.center,
              children: [
                const Spacer(),
                Text.rich(
                  context.l10n.errorScreenGenericDescription.toTextSpan(context),
                  textAlign: TextAlign.center,
                ),
                const Spacer(),
                ElevatedButton(
                  onPressed: () {
                    context.read<HistoryOverviewBloc>().add(const HistoryOverviewLoadTriggered());
                  },
                  child: Text.rich(context.l10n.generalRetry.toTextSpan(context)),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}
