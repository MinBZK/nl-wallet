import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/event/wallet_event.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/organization_policy.dart';
import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/object_extension.dart';
import '../../organization/detail/organization_detail_screen.dart';
import '../builder/request_detail_common_builders.dart';
import '../widget/button/bottom_back_button.dart';
import '../widget/divider_side.dart';
import '../widget/info_row.dart';
import '../widget/organization/organization_logo.dart';
import '../widget/sliver_wallet_app_bar.dart';
import '../widget/spacer/sliver_divider.dart';
import '../widget/spacer/sliver_sized_box.dart';
import '../widget/wallet_scrollbar.dart';

class RequestDetailsScreen extends StatelessWidget {
  /// Title: rendered at the top of the screen
  final String title;

  // Organization: rendered as a tappable organization info section when provided,
  // navigates to OrganizationDetailsScreen when clicked
  final Organization? organization;

  /// Request purpose: rendered as the reason for sharing section
  final LocalizedText? purpose;

  /// Requested attributes: rendered as a section with 'requested attrs.' header
  final List<WalletCard>? requestedAttributes;

  /// Shared attributes: rendered as a section with 'shared attrs.' header
  final List<WalletCard>? sharedAttributes;

  /// Policy: rendered as the agreement section.
  final OrganizationPolicy? policy;

  const RequestDetailsScreen({
    required this.title,
    this.organization,
    this.purpose,
    this.requestedAttributes,
    this.sharedAttributes,
    this.policy,
    super.key,
  }) : assert(
          sharedAttributes == null || requestedAttributes == null,
          'Only one of shared/requested attributes should be provided',
        );

  factory RequestDetailsScreen.forDisclosureEvent(String title, DisclosureEvent event) => RequestDetailsScreen(
        title: title,
        requestedAttributes: event.cards,
        purpose: event.purpose,
        organization: event.relyingParty,
        policy: OrganizationPolicy(organization: event.relyingParty, policy: event.policy),
      );

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: WalletScrollbar(
                child: CustomScrollView(
                  slivers: [
                    SliverWalletAppBar(
                      title: title,
                      scrollController: PrimaryScrollController.maybeOf(context),
                    ),
                    if (organization != null)
                      _buildOrganizationSliver(
                        context,
                        organization!,
                        DividerSide.top,
                      ),
                    if (purpose != null)
                      RequestDetailCommonBuilders.buildPurposeSliver(
                        context,
                        purpose: purpose!,
                        side: DividerSide.top,
                      ),
                    if (sharedAttributes != null)
                      RequestDetailCommonBuilders.buildSharedAttributesSliver(
                        context,
                        cards: sharedAttributes!,
                        side: DividerSide.top,
                      ),
                    if (requestedAttributes != null)
                      RequestDetailCommonBuilders.buildRequestedAttributesSliver(
                        context,
                        cards: requestedAttributes!,
                        side: DividerSide.top,
                      ),
                    if (policy != null)
                      RequestDetailCommonBuilders.buildPolicySliver(
                        context,
                        organization: policy!.organization,
                        policy: policy!.policy,
                        side: DividerSide.top,
                      ),
                    const SliverDivider(),
                    const SliverSizedBox(height: 24),
                  ],
                ),
              ),
            ),
            const BottomBackButton(),
          ],
        ),
      ),
    );
  }

  Widget _buildOrganizationSliver(BuildContext context, Organization organization, DividerSide side) {
    return SliverMainAxisGroup(
      slivers: [
        if (side.top) const SliverDivider(),
        SliverToBoxAdapter(
          child: InfoRow(
            padding: const EdgeInsets.symmetric(vertical: 16),
            leading: OrganizationLogo(image: organization.logo, size: 24),
            title: Text(
              context.l10n.requestDetailScreenAboutOrganizationCta(
                organization.displayName.l10nValue(context),
              ),
            ),
            subtitle: Text(organization.category?.l10nValue(context) ?? '').takeIf(
              (_) => organization.category != null,
            ),
            onTap: () => OrganizationDetailScreen.showPreloaded(
              context,
              organization,
              sharedDataWithOrganizationBefore: false,
            ),
          ),
        ),
        if (side.bottom) const SliverDivider(),
      ],
    );
  }

  static Future<dynamic> show(
    BuildContext context, {
    required String title,
    Organization? organization,
    LocalizedText? purpose,
    List<WalletCard>? requestedAttributes,
    List<WalletCard>? sharedAttributes,
    OrganizationPolicy? policy,
  }) {
    return Navigator.push(
      context,
      SecuredPageRoute(
        builder: (c) => RequestDetailsScreen(
          title: title,
          organization: organization,
          purpose: purpose,
          requestedAttributes: requestedAttributes,
          sharedAttributes: sharedAttributes,
          policy: policy,
        ),
      ),
    );
  }

  static Future<dynamic> showEvent(BuildContext context, DisclosureEvent event) {
    return Navigator.push(
      context,
      SecuredPageRoute(
        builder: (c) => RequestDetailsScreen.forDisclosureEvent(
          context.l10n.requestDetailScreenTitle,
          event,
        ),
      ),
    );
  }
}
