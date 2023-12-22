import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/timeline/timeline_attribute.dart';
import '../../../domain/model/timeline/timeline_section.dart';
import '../../../domain/model/wallet_card.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/timeline/timeline_section_list_factory.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/history/timeline_section_sliver.dart';
import '../../history/detail/argument/history_detail_screen_argument.dart';
import 'bloc/card_history_bloc.dart';

class CardHistoryScreen extends StatelessWidget {
  static String getArguments(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args as String;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in a (mock) id when opening the CardHistoryScreen');
    }
  }

  const CardHistoryScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(context.l10n.cardHistoryScreenTitle),
      ),
      body: SafeArea(
        child: _buildBody(context),
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    return BlocBuilder<CardHistoryBloc, CardHistoryState>(
      builder: (context, state) {
        return switch (state) {
          CardHistoryInitial() => _buildLoading(),
          CardHistoryLoadInProgress() => _buildLoading(),
          CardHistoryLoadSuccess() => _buildHistory(context, state),
          CardHistoryLoadFailure() => _buildError(context),
        };
      },
    );
  }

  Widget _buildLoading() => const CenteredLoadingIndicator();

  Widget _buildHistory(BuildContext context, CardHistoryLoadSuccess state) {
    final List<TimelineSection> sections = TimelineSectionListFactory.create(state.attributes);
    List<Widget> slivers = sections
        .map(
          (section) => TimelineSectionSliver(
            section: section,
            onRowPressed: (attribute) => _onTimelineRowPressed(context, attribute, state.card),
          ),
        )
        .toList();

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
          child: const BottomBackButton(
            showDivider: true,
          ),
        ),
      ],
    );
  }

  void _onTimelineRowPressed(BuildContext context, TimelineAttribute attribute, WalletCard card) {
    Navigator.pushNamed(
      context,
      WalletRoutes.historyDetailRoute,
      arguments: HistoryDetailScreenArgument(
        timelineAttribute: attribute,
        docType: card.docType,
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
              final settings = ModalRoute.of(context)?.settings;
              if (settings != null) {
                final cardId = getArguments(settings);
                context.read<CardHistoryBloc>().add(CardHistoryLoadTriggered(cardId));
              } else {
                Navigator.pop(context);
              }
            },
            child: Text(context.l10n.generalRetry),
          ),
          const SizedBox(height: 16),
        ],
      ),
    );
  }
}
