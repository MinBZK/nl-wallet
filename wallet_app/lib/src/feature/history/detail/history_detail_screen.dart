import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/event/wallet_event.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/wallet_event_extension.dart';
import '../../../util/formatter/history_details_time_formatter.dart';
import '../../../util/mapper/event/wallet_event_status_description_mapper.dart';
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
import '../../common/widget/sliver_wallet_app_bar.dart';
import '../../organization/detail/organization_detail_screen.dart';
import '../../policy/policy_screen.dart';
import 'argument/history_detail_screen_argument.dart';
import 'bloc/history_detail_bloc.dart';
import 'widget/history_detail_wallet_event_row.dart';

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
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: CustomScrollView(
                slivers: [
                  SliverWalletAppBar(title: context.l10n.historyDetailScreenTitle),
                  const SliverSizedBox(height: 8),
                  _buildBodySliver(context),
                ],
              ),
            ),
            _buildBottomBackButton(context),
          ],
        ),
      ),
    );
  }

  Widget _buildBodySliver(BuildContext context) {
    return BlocBuilder<HistoryDetailBloc, HistoryDetailState>(
      builder: (context, state) {
        return switch (state) {
          HistoryDetailInitial() => _buildLoadingSliver(),
          HistoryDetailLoadInProgress() => _buildLoadingSliver(),
          HistoryDetailLoadSuccess() => _buildSuccessSliver(context, state),
          HistoryDetailLoadFailure() => _buildErrorSliver(context),
        };
      },
    );
  }

  Widget _buildBottomBackButton(BuildContext context) {
    return BlocBuilder<HistoryDetailBloc, HistoryDetailState>(
      builder: (context, state) {
        return switch (state) {
          HistoryDetailInitial() => const BottomBackButton(),
          HistoryDetailLoadInProgress() => const BottomBackButton(),
          HistoryDetailLoadSuccess() => const BottomBackButton(),
          HistoryDetailLoadFailure() => const SizedBox.shrink(),
        };
      },
    );
  }

  Widget _buildLoadingSliver() {
    return const SliverFillRemaining(
      hasScrollBody: false,
      child: CenteredLoadingIndicator(),
    );
  }

  Widget _buildSuccessSliver(BuildContext context, HistoryDetailLoadSuccess state) {
    final WalletEvent event = state.event;
    final bool showStatusRow = _showStatusRow(event);
    final bool showDataAttributesSection = _showDataAttributesSection(event);
    final bool showContractSection = _showContractSection(event);
    final List<Widget> slivers = [];
    final Color iconColor = context.colorScheme.onSurfaceVariant;

    // Organization
    slivers.addAll([
      const SliverDivider(),
      SliverToBoxAdapter(
        child: _buildOrganizationRow(context, event),
      ),
      const SliverDivider(),
    ]);

    // Interaction request purpose
    if (event is DisclosureEvent && event.status == EventStatus.success) {
      slivers.addAll([
        SliverToBoxAdapter(
          child: _buildInteractionRequestPurposeRow(context, event.purpose),
        ),
        const SliverDivider(),
      ]);
    }

    // Interaction/operation type
    if (showStatusRow) {
      slivers.addAll([
        SliverToBoxAdapter(
          child: WalletEventStatusHeader(event: event),
        ),
        const SliverDivider(),
      ]);
    }

    // Data attributes
    if (showDataAttributesSection) {
      // Section title
      slivers.add(
        SliverToBoxAdapter(
          child: _buildDataAttributesSectionTitle(context, event),
        ),
      );

      // Signed contract (optional)
      if (showContractSection && event is SignEvent) {
        final signEvent = event;
        slivers.addAll([
          SliverToBoxAdapter(
            child: DocumentSection(
              padding: const EdgeInsets.only(left: 56, right: 16),
              document: signEvent.document,
              organization: signEvent.relyingParty,
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
                sourceCardTitle: event is DisclosureEvent ? entry.key.front.title.l10nValue(context) : null,
                attributes: entry.value,
              ),
            ),
          ),
        );
      }
      slivers.add(const SliverSizedBox(height: 16));

      // Policy section
      final Policy? policy = _getPolicyToDisplay(event);
      if (policy != null) {
        slivers.addAll(
          [
            const SliverDivider(),
            SliverToBoxAdapter(
              child: InfoRow(
                title: Text(context.l10n.historyDetailScreenTermsTitle),
                subtitle: Text(
                  context.l10n.historyDetailScreenTermsSubtitle(
                    event.relyingPartyOrIssuer.displayName.l10nValue(context),
                  ),
                ),
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
            onTap: () => PlaceholderScreen.showGeneric(context),
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
            onTap: () => PlaceholderScreen.showGeneric(context),
          ),
        ),
      );
      slivers.add(const SliverDivider());
      slivers.add(const SliverSizedBox(height: 24));
    }

    return SliverMainAxisGroup(slivers: slivers);
  }

  Widget _buildInteractionRequestPurposeRow(BuildContext context, LocalizedText purpose) {
    return InfoRow(
      title: Text(context.l10n.historyDetailScreenInteractionRequestPurposeTitle),
      subtitle: Text(purpose.l10nValue(context)),
      leading: Icon(
        Icons.outlined_flag_outlined,
        color: context.colorScheme.onSurfaceVariant,
      ),
    );
  }

  Widget _buildOrganizationRow(BuildContext context, WalletEvent event) {
    final organization = event.relyingPartyOrIssuer;
    final status = _getOrganizationNamePrefixBasedOnStatus(context, event);
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

  String _getOrganizationNamePrefixBasedOnStatus(BuildContext context, WalletEvent event) {
    return switch (event) {
      DisclosureEvent() => event.status == EventStatus.success
          ? context.l10n.historyDetailScreenOrganizationNamePrefixInteractionStatusSuccess
          : context.l10n.historyDetailScreenOrganizationNamePrefixInteractionStatusNonSuccess,
      IssuanceEvent() => context.l10n.historyDetailScreenOrganizationNamePrefixOperationStatusAll,
      SignEvent() => context.l10n.historyDetailScreenOrganizationNamePrefixSigningStatusAll,
    };
  }

  bool _showStatusRow(WalletEvent event) {
    return switch (event) {
      DisclosureEvent() => event.status != EventStatus.success,
      IssuanceEvent() => false,
      SignEvent() => event.status != EventStatus.success,
    };
  }

  bool _showDataAttributesSection(WalletEvent event) {
    return switch (event) {
      DisclosureEvent() => event.status == EventStatus.success,
      IssuanceEvent() => true,
      SignEvent() => event.status == EventStatus.success,
    };
  }

  bool _showContractSection(WalletEvent event) => event is SignEvent && event.status == EventStatus.success;

  Policy? _getPolicyToDisplay(WalletEvent event) {
    return switch (event) {
      DisclosureEvent(status: var status) when status == EventStatus.success => event.policy,
      DisclosureEvent() => null,
      IssuanceEvent() => null,
      SignEvent(status: var status) when status == EventStatus.success => event.policy,
      SignEvent() => null,
    };
  }

  Widget _buildDataAttributesSectionTitle(BuildContext context, WalletEvent event) {
    // Declare variables / defaults
    final iconColor = context.colorScheme.onSurfaceVariant;
    String title = '';
    String subtitle = '';
    Widget icon = Image.asset(WalletAssets.icon_card_share, color: iconColor);

    switch (event) {
      case DisclosureEvent():
        title = context.l10n.historyDetailScreenInteractionAttributesTitle(event.attributes.length);
      case IssuanceEvent():
        title = event.card.front.title.l10nValue(context);
        subtitle = WalletEventStatusDescriptionMapper().map(context, event);
        icon = Icon(Icons.credit_card_outlined, color: iconColor);
      case SignEvent():
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
                    HistoryDetailsTimeFormatter.format(context, event.dateTime),
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

  Widget _buildErrorSliver(BuildContext context) {
    return SliverFillRemaining(
      hasScrollBody: false,
      child: Padding(
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
                          event: args.walletEvent,
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
      ),
    );
  }
}
