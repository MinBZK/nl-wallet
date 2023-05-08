import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../../domain/model/timeline/signing_timeline_attribute.dart';
import '../../../domain/model/timeline/timeline_attribute.dart';
import '../../../util/formatter/history_details_time_formatter.dart';
import '../../common/widget/attribute/data_attribute_section.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/document_section.dart';
import '../../common/widget/info_row.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../common/widget/sliver_divider.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../organization/detail/organization_detail_screen.dart';
import '../../organization/widget/organization_row.dart';
import '../../policy/policy_screen.dart';
import 'argument/history_detail_screen_argument.dart';
import 'bloc/history_detail_bloc.dart';
import 'widget/history_detail_timeline_attribute_row.dart';

class HistoryDetailScreen extends StatelessWidget {
  static HistoryDetailScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return HistoryDetailScreenArgument.fromMap(args as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in [HistoryDetailScreenArgument] when opening the HistoryDetailScreen');
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
    final locale = AppLocalizations.of(context);
    final TimelineAttribute timelineAttribute = state.timelineAttribute;
    final bool showTimelineStatusRow = _showTimelineStatusRow(timelineAttribute);
    final bool showDataAttributesSection = _showDataAttributesSection(timelineAttribute);
    final bool showContractSection = _showContractSection(timelineAttribute);
    final List<Widget> slivers = [];

    // Header
    slivers.addAll([
      SliverToBoxAdapter(
        child: OrganizationRow(
          onTap: () => OrganizationDetailScreen.show(
            context,
            timelineAttribute.organization.id,
            timelineAttribute.organization.shortName,
          ),
          organizationName: timelineAttribute.organization.shortName,
          logoUrl: timelineAttribute.organization.logoUrl,
        ),
      ),
      const SliverDivider(),
    ]);

    // Interaction/operation type
    if (showTimelineStatusRow) {
      slivers.addAll([
        SliverToBoxAdapter(
          child: HistoryDetailTimelineAttributeRow(
            attribute: timelineAttribute,
          ),
        ),
        const SliverDivider(),
      ]);
    }

    // Data attributes
    if (showDataAttributesSection) {
      // Section title
      slivers.add(
        SliverToBoxAdapter(
          child: _buildDataAttributesSectionTitle(context, timelineAttribute),
        ),
      );

      // Signed contract (optional)
      if (showContractSection) {
        final signingAttribute = (timelineAttribute as SigningTimelineAttribute);
        slivers.addAll([
          SliverToBoxAdapter(
            child: DocumentSection(
              padding: const EdgeInsets.only(left: 56, right: 16),
              document: signingAttribute.document,
              organization: signingAttribute.organization,
            ),
          ),
          const SliverDivider(height: 32),
          const SliverSizedBox(height: 8),
        ]);
      }

      // Data attributes
      for (final entry in state.attributesByCard.entries) {
        slivers.add(
          SliverToBoxAdapter(
            child: Padding(
              padding: const EdgeInsets.only(left: 56, bottom: 8, right: 16),
              child: DataAttributeSection(
                sourceCardTitle: entry.key.front.title,
                attributes: entry.value,
              ),
            ),
          ),
        );
      }
      slivers.add(const SliverSizedBox(height: 16));
      slivers.add(const SliverDivider());

      // Policy section
      final Policy? policy = _getPolicyToDisplay(timelineAttribute);
      if (policy != null) {
        slivers.add(
          SliverToBoxAdapter(
            child: InfoRow(
              title: locale.historyDetailScreenTermsTitle,
              subtitle: locale.historyDetailScreenTermsSubtitle,
              leading: Icon(Icons.policy_outlined, color: Theme.of(context).colorScheme.primary),
              onTap: () => PolicyScreen.show(context, policy),
            ),
          ),
        );
      }

      // Incorrect button
      slivers.add(const SliverDivider());
      slivers.add(
        SliverToBoxAdapter(
          child: InfoRow(
            title: locale.historyDetailScreenIssueTitle,
            subtitle: locale.historyDetailScreenIssueSubtitle,
            leading: Icon(Icons.gpp_maybe_outlined, color: Theme.of(context).colorScheme.primary),
            onTap: () => PlaceholderScreen.show(context),
          ),
        ),
      );
      slivers.add(const SliverDivider());
      slivers.add(
        SliverToBoxAdapter(
          child: InfoRow(
            title: locale.historyDetailScreenHelpdeskTitle,
            subtitle: locale.historyDetailScreenHelpdeskSubtitle,
            leading: Icon(Icons.comment_outlined, color: Theme.of(context).colorScheme.primary),
            onTap: () => PlaceholderScreen.show(context),
          ),
        ),
      );
      slivers.add(const SliverDivider());
      slivers.add(const SliverSizedBox(height: 32));
    }

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
        const BottomBackButton(showDivider: true),
      ],
    );
  }

  bool _showTimelineStatusRow(TimelineAttribute attribute) {
    if (attribute is InteractionTimelineAttribute) {
      return attribute.status != InteractionStatus.success;
    } else if (attribute is SigningTimelineAttribute) {
      return attribute.status != SigningStatus.success;
    }
    return true;
  }

  bool _showDataAttributesSection(TimelineAttribute attribute) {
    if (attribute is InteractionTimelineAttribute) {
      return attribute.status == InteractionStatus.success;
    } else if (attribute is SigningTimelineAttribute) {
      return attribute.status == SigningStatus.success;
    }
    return true;
  }

  bool _showContractSection(TimelineAttribute timelineAttribute) {
    return (timelineAttribute is SigningTimelineAttribute) && timelineAttribute.status == SigningStatus.success;
  }

  Policy? _getPolicyToDisplay(TimelineAttribute timelineAttribute) {
    if (timelineAttribute is InteractionTimelineAttribute && timelineAttribute.status == InteractionStatus.success) {
      return timelineAttribute.policy;
    } else if (timelineAttribute is SigningTimelineAttribute && timelineAttribute.status == SigningStatus.success) {
      return timelineAttribute.policy;
    }
    return null;
  }

  Widget _buildDataAttributesSectionTitle(BuildContext context, TimelineAttribute attribute) {
    final locale = AppLocalizations.of(context);

    String title = '';
    if (attribute is InteractionTimelineAttribute) {
      title = locale.historyDetailScreenInteractionAttributesTitle(attribute.dataAttributes.length);
    } else if (attribute is OperationTimelineAttribute) {
      title = locale.historyDetailScreenOperationAttributesTitle;
    } else if (attribute is SigningTimelineAttribute) {
      title = locale.historyDetailScreenSigningAttributesTitle;
    }

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          SizedBox(
            height: 24,
            width: 24,
            child: Image.asset('assets/images/ic_card_share.png'),
          ),
          const SizedBox(width: 16),
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                title,
                style: Theme.of(context).textTheme.displaySmall,
              ),
              const SizedBox(height: 2),
              Text(
                HistoryDetailsTimeFormatter.format(locale, attribute.dateTime),
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
          ),
        ],
      ),
    );
  }
}
