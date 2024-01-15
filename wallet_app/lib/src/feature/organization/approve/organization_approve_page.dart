import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/confirm_buttons.dart';
import '../../common/widget/button/link_tile_button.dart';
import '../../common/widget/organization/organization_logo.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../detail/organization_detail_screen.dart';

class OrganizationApprovePage extends StatelessWidget {
  /// Callback that is triggered when the user approves the request
  final VoidCallback onAcceptPressed;

  /// Callback that is triggered when the user declines the request
  final VoidCallback onDeclinePressed;

  /// Callback that is triggered when the user wants to report an issue
  final VoidCallback? onReportIssuePressed;

  /// The organization that user is interacting with
  final Organization organization;

  /// The organization that user is interacting with
  final String originUrl;

  /// Tells the Page in which flow it's currently used, used to select the correct string resources
  final ApprovalPurpose purpose;

  /// If true, the 'first interaction' banner will be shown. //FIXME: This should eventually be a interactionCount
  final bool isFirstInteractionWithOrganization;

  const OrganizationApprovePage({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    required this.organization,
    required this.originUrl,
    required this.purpose,
    this.onReportIssuePressed,
    this.isFirstInteractionWithOrganization = true,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      child: CustomScrollView(
        restorationId: 'approve_organization_scrollview',
        slivers: <Widget>[
          const SliverSizedBox(height: 16),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          const SliverSizedBox(height: 32),
          SliverToBoxAdapter(
            child: LinkTileButton(
              child: Text(context.l10n.organizationApprovePageMoreInfoCta),
              onPressed: () => _openOrganizationDetails(context),
            ),
          ),
          const SliverSizedBox(height: 32),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: Container(
              alignment: Alignment.bottomCenter,
              child: ConfirmButtons(
                forceVertical: true,
                onAcceptPressed: onAcceptPressed,
                acceptIcon: Icons.arrow_forward,
                acceptText: _approveButtonText(context),
                onDeclinePressed: onDeclinePressed,
                declineText: _declineButtonText(context),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: MergeSemantics(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            OrganizationLogo(
              image: organization.logo,
              size: 64,
              fixedRadius: 12,
            ),
            const SizedBox(height: 24),
            _buildHeaderTitleText(context),
            const SizedBox(height: 8),
            _buildFraudInfoText(context),
          ],
        ),
      ),
    );
  }

  Widget _buildHeaderTitleText(BuildContext context) {
    return Text(
      context.l10n.organizationApprovePageGenericTitle(organization.legalName.l10nValue(context)),
      style: context.textTheme.displayMedium,
      textAlign: TextAlign.start,
    );
  }

  Widget _buildFraudInfoText(BuildContext context) {
    final fullString = context.l10n.organizationApprovePageFraudInfo(originUrl);
    final parts = fullString.split(originUrl);

    /// We only support the case where the url is somewhere inside the fullString, e.g. "Open {url} for more info"
    if (parts.length != 2) return Text(fullString, style: context.textTheme.bodyLarge);
    return RichText(
      textAlign: TextAlign.start,
      text: TextSpan(
        style: context.textTheme.bodyLarge,
        children: [
          TextSpan(text: parts.first),
          TextSpan(
            text: originUrl,
            style: TextStyle(
              color: context.colorScheme.primary,
              decoration: TextDecoration.underline,
            ),
          ),
          TextSpan(text: parts.last),
        ],
      ),
    );
  }

  String _approveButtonText(BuildContext context) {
    switch (purpose) {
      case ApprovalPurpose.issuance:
        return context.l10n.organizationApprovePageApproveCta;
      case ApprovalPurpose.disclosure:
        return context.l10n.organizationApprovePageShareWithApproveCta;
      case ApprovalPurpose.sign:
        return context.l10n.organizationApprovePageApproveCta;
    }
  }

  String _declineButtonText(BuildContext context) {
    switch (purpose) {
      case ApprovalPurpose.issuance:
        return context.l10n.organizationApprovePageDenyCta;
      case ApprovalPurpose.disclosure:
        return context.l10n.organizationApprovePageShareWithDenyCta;
      case ApprovalPurpose.sign:
        return context.l10n.organizationApprovePageDenyCta;
    }
  }

  void _openOrganizationDetails(BuildContext context) {
    OrganizationDetailScreen.showPreloaded(
      context,
      organization,
      isFirstInteractionWithOrganization,
      onReportIssuePressed: onReportIssuePressed,
    );
  }
}

enum ApprovalPurpose { issuance, disclosure, sign }
