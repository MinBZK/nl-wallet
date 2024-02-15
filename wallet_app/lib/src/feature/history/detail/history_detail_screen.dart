import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../../domain/model/timeline/signing_timeline_attribute.dart';
import '../../../domain/model/timeline/timeline_attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/formatter/history_details_time_formatter.dart';
import '../../../util/formatter/timeline_attribute_status_description_text_formatter.dart';
import '../../../wallet_assets.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/attribute/data_attribute_section.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/document_section.dart';
import '../../common/widget/info_row.dart';
import '../../common/widget/organization/organization_logo.dart';
import '../../common/widget/sliver_divider.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../common/widget/wallet_app_bar.dart';
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

  const HistoryDetailScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: Text(context.l10n.historyDetailScreenTitle),
      ),
      body: SafeArea(
        child: _buildBody(context),
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    return BlocBuilder<HistoryDetailBloc, HistoryDetailState>(
      builder: (context, state) {
        return switch (state) {
          HistoryDetailInitial() => _buildLoading(),
          HistoryDetailLoadInProgress() => _buildLoading(),
          HistoryDetailLoadSuccess() => _buildSuccess(context, state),
          HistoryDetailLoadFailure() => _buildError(context),
        };
      },
    );
  }

  Widget _buildLoading() {
    return const CenteredLoadingIndicator();
  }

  Widget _buildSuccess(BuildContext context, HistoryDetailLoadSuccess state) {
    final TimelineAttribute attribute = state.timelineAttribute;
    final bool showTimelineStatusRow = _showTimelineStatusRow(attribute);
    final bool showDataAttributesSection = _showDataAttributesSection(attribute);
    final bool showContractSection = _showContractSection(attribute);
    final List<Widget> slivers = [];
    final Color iconColor = context.colorScheme.onSurface;

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
                sourceCardTitle:
                    attribute is InteractionTimelineAttribute ? entry.key.front.title.l10nValue(context) : null,
                attributes: entry.value,
              ),
            ),
          ),
        );
      }
      slivers.add(const SliverSizedBox(height: 16));

      // Policy section
      final Policy? policy = _getPolicyToDisplay(attribute);
      if (policy != null) {
        slivers.addAll(
          [
            const SliverDivider(),
            SliverToBoxAdapter(
              child: InfoRow(
                title: Text(context.l10n.historyDetailScreenTermsTitle),
                subtitle: Text(context.l10n
                    .historyDetailScreenTermsSubtitle(attribute.organization.displayName.l10nValue(context))),
                leading: Icon(Icons.policy_outlined, color: iconColor),
                onTap: () => PolicyScreen.show(context, policy),
              ),
            ),
          ],
        );
      }

      // Report issue button
      slivers.add(const SliverDivider());
      slivers.add(
        SliverToBoxAdapter(
          child: InfoRow(
            title: Text(context.l10n.historyDetailScreenIssueTitle),
            subtitle: Text(context.l10n.historyDetailScreenIssueSubtitle),
            leading: Icon(Icons.gpp_maybe_outlined, color: iconColor),
            onTap: () => PlaceholderScreen.show(context),
          ),
        ),
      );

      // Helpdesk button
      slivers.add(const SliverDivider());
      slivers.add(
        SliverToBoxAdapter(
          child: InfoRow(
            title: Text(context.l10n.historyDetailScreenHelpdeskTitle),
            subtitle: Text(context.l10n.historyDetailScreenHelpdeskSubtitle),
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
            child: CustomScrollView(
              slivers: slivers,
            ),
          ),
        ),
        const BottomBackButton(),
      ],
    );
  }

  Widget _buildInteractionRequestPurposeRow(BuildContext context, InteractionTimelineAttribute attribute) {
    return InfoRow(
      title: Text(context.l10n.historyDetailScreenInteractionRequestPurposeTitle),
      subtitle: Text(attribute.requestPurpose.l10nValue(context)),
      leading: Icon(
        Icons.outlined_flag_outlined,
        color: context.colorScheme.onSurface,
      ),
    );
  }

  Widget _buildOrganizationRow(BuildContext context, TimelineAttribute attribute) {
    final organization = attribute.organization;
    final status = _getOrganizationNamePrefixBasedOnStatus(context, attribute);
    return InfoRow(
      leading: ExcludeSemantics(
        child: OrganizationLogo(
          image: organization.logo,
          size: _kOrganizationLogoSize,
        ),
      ),
      title: Text(context.l10n.historyDetailScreenOrganizationNameAndStatus(
        organization.displayName.l10nValue(context),
        status,
      )),
      subtitle: Text(organization.category?.l10nValue(context) ?? ''),
      onTap: () => OrganizationDetailScreen.showPreloaded(context, organization, false),
    );
  }

  String _getOrganizationNamePrefixBasedOnStatus(BuildContext context, TimelineAttribute attribute) {
    if (attribute is InteractionTimelineAttribute) {
      return attribute.status == InteractionStatus.success
          ? context.l10n.historyDetailScreenOrganizationNamePrefixInteractionStatusSuccess
          : context.l10n.historyDetailScreenOrganizationNamePrefixInteractionStatusNonSuccess;
    } else if (attribute is OperationTimelineAttribute) {
      return context.l10n.historyDetailScreenOrganizationNamePrefixOperationStatusAll;
    } else if (attribute is SigningTimelineAttribute) {
      return context.l10n.historyDetailScreenOrganizationNamePrefixSigningStatusAll;
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
    final iconColor = context.colorScheme.onSurface;

    String title = '';
    String subtitle = '';
    Widget icon = Image.asset(
      WalletAssets.icon_card_share,
      color: iconColor,
    );

    if (attribute is InteractionTimelineAttribute) {
      title = context.l10n.historyDetailScreenInteractionAttributesTitle(attribute.dataAttributes.length);
    } else if (attribute is OperationTimelineAttribute) {
      title = attribute.card.front.title.l10nValue(context);
      subtitle = TimelineAttributeStatusDescriptionTextFormatter.map(context, attribute);
      icon = Icon(
        Icons.credit_card_outlined,
        color: iconColor,
      );
    } else if (attribute is SigningTimelineAttribute) {
      title = context.l10n.historyDetailScreenSigningAttributesTitle;
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
          Expanded(
            child: MergeSemantics(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    title,
                    style: context.textTheme.displaySmall,
                  ),
                  const SizedBox(height: 2),
                  if (subtitle.isNotEmpty) ...[
                    Text(
                      subtitle,
                      style: context.textTheme.bodyLarge,
                    ),
                    const SizedBox(height: 2),
                  ],
                  Text(
                    HistoryDetailsTimeFormatter.format(context, attribute.dateTime),
                    style: context.textTheme.bodySmall,
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
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
              var settings = ModalRoute.of(context)?.settings;
              if (settings != null) {
                final args = getArgument(settings);
                context.read<HistoryDetailBloc>().add(
                      HistoryDetailLoadTriggered(
                        attribute: args.timelineAttribute,
                        docType: args.docType,
                      ),
                    );
              } else {
                Navigator.pop(context);
              }
            },
            child: Text(context.l10n.generalRetry),
          ),
        ],
      ),
    );
  }
}
