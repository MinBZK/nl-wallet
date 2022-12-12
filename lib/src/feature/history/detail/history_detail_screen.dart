import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/timeline_attribute.dart';
import '../../common/widget/attribute/data_attribute_row.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/link_button.dart';
import '../../common/widget/placeholder_screen.dart';
import 'bloc/history_detail_bloc.dart';
import 'widget/history_detail_header.dart';
import 'widget/history_detail_timeline_attribute_row.dart';

class HistoryDetailScreen extends StatelessWidget {
  static String getArguments(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return args as String;
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args (type: ${args.runtimeType})', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in an attributeId.');
    }
  }

  const HistoryDetailScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(AppLocalizations.of(context).historyDetailScreenTitle),
      ),
      body: _buildBody(context),
    );
  }

  Widget _buildBody(BuildContext context) {
    return BlocBuilder<HistoryDetailBloc, HistoryDetailState>(
      builder: (context, state) {
        if (state is HistoryDetailInitial) return _buildLoading();
        if (state is HistoryDetailLoadInProgress) return _buildLoading();
        if (state is HistoryDetailLoadSuccess) return _buildSuccess(context, state);
        throw UnsupportedError('Unknown state: $state');
      },
    );
  }

  Widget _buildLoading() {
    return const CenteredLoadingIndicator();
  }

  Widget _buildSuccess(BuildContext context, HistoryDetailLoadSuccess state) {
    final TimelineAttribute attribute = state.attribute;
    final bool showTimelineTypeRow = _showTimelineTypeRow(attribute);
    final List<Widget> slivers = [];

    // Header
    slivers.addAll([
      SliverToBoxAdapter(
        child: HistoryDetailHeader(
          organization: attribute.organization,
          dateTime: attribute.dateTime,
        ),
      ),
      const SliverToBoxAdapter(child: Divider(height: 1)),
    ]);

    // Interaction/operation type
    if (showTimelineTypeRow) {
      slivers.addAll([
        SliverToBoxAdapter(child: HistoryDetailTimelineAttributeRow(attribute: attribute)),
        const SliverToBoxAdapter(child: Divider(height: 1)),
      ]);
    }

    // Data attributes
    final List<DataAttribute> dataAttributes = attribute.attributes;
    if (dataAttributes.isNotEmpty) {
      // Section title
      slivers.add(SliverToBoxAdapter(child: _buildDataAttributesSectionTitle(context, attribute)));

      // Data attributes
      for (DataAttribute dataAttribute in dataAttributes) {
        slivers.add(SliverToBoxAdapter(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: DataAttributeRow(attribute: dataAttribute),
          ),
        ));
      }

      // Incorrect button
      slivers.add(const SliverToBoxAdapter(child: Divider(height: 32)));
      slivers.add(SliverToBoxAdapter(child: _buildIncorrectButton(context)));
      slivers.add(const SliverToBoxAdapter(child: Divider(height: 32)));
    }

    return CustomScrollView(slivers: slivers);
  }

  bool _showTimelineTypeRow(TimelineAttribute attribute) {
    if (attribute is InteractionAttribute) {
      return attribute.interactionType != InteractionType.success;
    }
    return true;
  }

  Widget _buildDataAttributesSectionTitle(BuildContext context, TimelineAttribute attribute) {
    final locale = AppLocalizations.of(context);

    String title = '';
    if (attribute is InteractionAttribute) {
      title = locale.historyDetailScreenInteractionAttributesTitle;
    } else if (attribute is OperationAttribute) {
      title = locale.historyDetailScreenOperationAttributesTitle;
    }

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            title,
            style: Theme.of(context).textTheme.headline3,
          ),
        ],
      ),
    );
  }

  Widget _buildIncorrectButton(BuildContext context) {
    final buttonText = AppLocalizations.of(context).cardDataScreenIncorrectCta;
    return Align(
      alignment: AlignmentDirectional.centerStart,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8.0),
        child: LinkButton(
          child: Text(buttonText),
          onPressed: () => PlaceholderScreen.show(context, buttonText),
        ),
      ),
    );
  }
}
