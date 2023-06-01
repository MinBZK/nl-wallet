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
import '../../../util/mapper/timeline_attribute_status_description_text_mapper.dart';
import '../../common/widget/attribute/data_attribute_section.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/document_section.dart';
import '../../common/widget/info_row.dart';
import '../../common/widget/organization/organization_logo.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../common/widget/sliver_divider.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../organization/detail/organization_detail_screen.dart';
import '../../policy/policy_screen.dart';
import 'argument/history_detail_screen_argument.dart';
import 'bloc/history_detail_bloc.dart';
import 'widget/history_detail_timeline_attribute_row.dart';

const _kOrganizationLogoSize = 24.0;

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
      body: SafeArea(
        child: _buildBody(context),
      ),
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
    final TimelineAttribute attribute = state.timelineAttribute;
    final bool showTimelineStatusRow = _showTimelineStatusRow(attribute);
    final bool showDataAttributesSection = _showDataAttributesSection(attribute);
    final bool showContractSection = _showContractSection(attribute);
    final List<Widget> slivers = [];
    final Color iconColor = Theme.of(context).colorScheme.onSurface;

    slivers.addAll([
      // Organization
      SliverToBoxAdapter(
        child: _buildOrganizationRow(context, attribute),
      ),
      const SliverDivider(),
    ]);

    // Interaction request purpose
    if (attribute is InteractionTimelineAttribute && attribute.status == InteractionStatus.success) {
      slivers.addAll([
        SliverToBoxAdapter(
          child: _buildInteractionRequestPurposeRow(context, attribute),
        ),
        const SliverDivider(),
      ]);
    }

    // Interaction/operation type
    if (showTimelineStatusRow) {
      slivers.addAll([
        SliverToBoxAdapter(
          child: HistoryDetailTimelineAttributeRow(
            attribute: attribute,
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
          child: _buildDataAttributesSectionTitle(context, attribute),
        ),
      );

      // Signed contract (optional)
      if (showContractSection) {
        final signingAttribute = (attribute as SigningTimelineAttribute);
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
                sourceCardTitle: attribute is InteractionTimelineAttribute ? entry.key.front.title : null,
                attributes: entry.value,
              ),
            ),
          ),
        );
      }
      slivers.add(const SliverSizedBox(height: 16));
      slivers.add(const SliverDivider());

      // Policy section
      final Policy? policy = _getPolicyToDisplay(attribute);
      if (policy != null) {
        slivers.add(
          SliverToBoxAdapter(
            child: InfoRow(
              title: locale.historyDetailScreenTermsTitle,
              subtitle: locale.historyDetailScreenTermsSubtitle(attribute.organization.shortName),
              leading: Icon(Icons.policy_outlined, color: iconColor),
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
            leading: Icon(Icons.gpp_maybe_outlined, color: iconColor),
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
            leading: Icon(Icons.comment_outlined, color: iconColor),
            onTap: () => PlaceholderScreen.show(context),
          ),
        ),
      );
      slivers.add(const SliverDivider());
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

  Widget _buildInteractionRequestPurposeRow(BuildContext context, InteractionTimelineAttribute attribute) {
    return InfoRow(
      title: AppLocalizations.of(context).historyDetailScreenInteractionRequestPurposeTitle,
      subtitle: attribute.requestPurpose,
      leading: Icon(
        Icons.outlined_flag_outlined,
        color: Theme.of(context).colorScheme.onSurface,
      ),
    );
  }

  Widget _buildOrganizationRow(BuildContext context, TimelineAttribute attribute) {
    final locale = AppLocalizations.of(context);
    final organization = attribute.organization;
    final status = _getTimelineAttributeStatus(context, attribute);
    return InfoRow(
      leading: OrganizationLogo(
        image: AssetImage(organization.logoUrl),
        size: _kOrganizationLogoSize,
      ),
      title: locale.historyDetailScreenOrganizationNameAndStatus(
        status,
        organization.shortName,
      ),
      subtitle: organization.category,
      onTap: () => OrganizationDetailScreen.show(
        context,
        organization.id,
        organization.shortName,
      ),
    );
  }

  String _getTimelineAttributeStatus(BuildContext context, TimelineAttribute attribute) {
    final locale = AppLocalizations.of(context);
    if (attribute is InteractionTimelineAttribute) {
      return attribute.status == InteractionStatus.success
          ? locale.historyDetailScreenOrganizationNamePrefixInteractionStatusSuccess
          : locale.historyDetailScreenOrganizationNamePrefixInteractionStatusNonSuccess;
    } else if (attribute is OperationTimelineAttribute) {
      return locale.historyDetailScreenOrganizationNamePrefixOperationStatusAll;
    } else if (attribute is SigningTimelineAttribute) {
      return locale.historyDetailScreenOrganizationNamePrefixSigningStatusAll;
    }
    return '';
  }

  bool _showTimelineStatusRow(TimelineAttribute attribute) {
    if (attribute is InteractionTimelineAttribute) {
      return attribute.status != InteractionStatus.success;
    } else if (attribute is SigningTimelineAttribute) {
      return attribute.status != SigningStatus.success;
    }
    return false;
  }

  bool _showDataAttributesSection(TimelineAttribute attribute) {
    if (attribute is InteractionTimelineAttribute) {
      return attribute.status == InteractionStatus.success;
    } else if (attribute is SigningTimelineAttribute) {
      return attribute.status == SigningStatus.success;
    }
    return true;
  }

  bool _showContractSection(TimelineAttribute attribute) {
    return (attribute is SigningTimelineAttribute) && attribute.status == SigningStatus.success;
  }

  Policy? _getPolicyToDisplay(TimelineAttribute attribute) {
    if (attribute is InteractionTimelineAttribute && attribute.status == InteractionStatus.success) {
      return attribute.policy;
    } else if (attribute is SigningTimelineAttribute && attribute.status == SigningStatus.success) {
      return attribute.policy;
    }
    return null;
  }

  Widget _buildDataAttributesSectionTitle(BuildContext context, TimelineAttribute attribute) {
    final locale = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final iconColor = theme.colorScheme.onSurface;

    String title = '';
    String subtitle = '';
    Widget icon = Image.asset(
      'assets/images/ic_card_share.png',
      color: iconColor,
    );

    if (attribute is InteractionTimelineAttribute) {
      title = locale.historyDetailScreenInteractionAttributesTitle(attribute.dataAttributes.length);
    } else if (attribute is OperationTimelineAttribute) {
      title = attribute.cardTitle;
      subtitle = TimelineAttributeStatusDescriptionTextMapper.map(locale, attribute);
      icon = Icon(
        Icons.credit_card_outlined,
        color: iconColor,
      );
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
            child: icon,
          ),
          const SizedBox(width: 16),
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                title,
                style: theme.textTheme.displaySmall,
              ),
              const SizedBox(height: 2),
              if (subtitle.isNotEmpty) ...[
                Text(
                  subtitle,
                  style: theme.textTheme.bodyLarge,
                ),
                const SizedBox(height: 2),
              ],
              Text(
                HistoryDetailsTimeFormatter.format(locale, attribute.dateTime),
                style: theme.textTheme.bodySmall,
              ),
            ],
          ),
        ],
      ),
    );
  }
}
