import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/disclosure/disclosure_session_type.dart';
import '../../../domain/model/organization.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/confirm/confirm_button.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/link_tile_button.dart';
import '../../common/widget/organization/organization_logo.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../common/widget/text_with_link.dart';
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

  /// The url from which the user should have opened the flow. Prominently displayed for the user to check
  final String originUrl;

  /// Tells the Page in which flow it's currently used, used to select the correct string resources
  final ApprovalPurpose purpose;

  /// If true, the 'first interaction' banner will be shown
  final bool sharedDataWithOrganizationBefore;

  /// If `crossDevice`, the 'fraud warning' (including `originUrl`) will be shown
  final DisclosureSessionType? sessionType;

  const OrganizationApprovePage({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    required this.organization,
    required this.originUrl,
    required this.purpose,
    this.onReportIssuePressed,
    this.sharedDataWithOrganizationBefore = false,
    this.sessionType,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Scrollbar(
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
              child: Column(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  const Divider(height: 1),
                  ConfirmButtons(
                    primaryButton: ConfirmButton.accept(
                      onPressed: onAcceptPressed,
                      text: _approveButtonText(context),
                      icon: Icons.arrow_forward,
                    ),
                    secondaryButton: ConfirmButton.reject(
                      onPressed: onDeclinePressed,
                      text: _declineButtonText(context),
                      icon: Icons.block_flipped,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
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
            if (sessionType == DisclosureSessionType.crossDevice) ...[
              const SizedBox(height: 8),
              _buildFraudInfoText(context),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildHeaderTitleText(BuildContext context) {
    return Text(
      context.l10n.organizationApprovePageGenericTitle(organization.displayName.l10nValue(context)),
      style: context.textTheme.displayMedium,
      textAlign: TextAlign.start,
    );
  }

  Widget _buildFraudInfoText(BuildContext context) {
    final fullString = context.l10n.organizationApprovePageFraudInfo(originUrl);
    return TextWithLink(fullText: fullString, ctaText: originUrl);
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
      sharedDataWithOrganizationBefore,
      onReportIssuePressed: onReportIssuePressed,
    );
  }
}

enum ApprovalPurpose { issuance, disclosure, sign }
