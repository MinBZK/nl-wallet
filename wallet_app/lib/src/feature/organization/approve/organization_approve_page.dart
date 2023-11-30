import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/widget/button/confirm_buttons.dart';
import '../../common/widget/icon_row.dart';
import '../../common/widget/organization/organization_logo.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../detail/organization_detail_screen.dart';
import '../widget/organization_row.dart';

class OrganizationApprovePage extends StatelessWidget {
  /// Callback that is triggered when the user approves the request
  final VoidCallback onAcceptPressed;

  /// Callback that is triggered when the user declines the request
  final VoidCallback onDeclinePressed;

  /// Callback that is triggered when the user wants to report an issue
  final VoidCallback? onReportIssuePressed;

  /// The organization that user is interacting with
  final Organization organization;

  /// Tells the Page in which flow it's currently used, used to select the correct string resources
  final ApprovalPurpose purpose;

  /// Inform the user what the purpose is of this request
  final LocalizedText? requestPurpose;

  /// If true, the 'first interaction' banner will be shown.
  final bool isFirstInteractionWithOrganization;

  const OrganizationApprovePage({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    required this.organization,
    required this.purpose,
    this.requestPurpose,
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
          const SliverSizedBox(height: 32),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          const SliverSizedBox(height: 12),
          SliverToBoxAdapter(child: _buildInfoRows(context)),
          const SliverSizedBox(height: 12),
          const SliverToBoxAdapter(child: Divider(height: 1)),
          SliverToBoxAdapter(
            child: OrganizationRow(
              onTap: () => OrganizationDetailScreen.showPreloaded(
                context,
                organization,
                isFirstInteractionWithOrganization,
                onReportIssuePressed: onReportIssuePressed,
              ),
              subtitle: organization.category?.l10nValue(context) ?? '',
            ),
          ),
          const SliverToBoxAdapter(child: Divider(height: 1)),
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
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          OrganizationLogo(image: organization.logo, size: 64),
          const SizedBox(height: 24),
          Text(
            _headerTitleText(context),
            style: context.textTheme.displayMedium,
            textAlign: TextAlign.start,
          ),
        ],
      ),
    );
  }

  String _headerTitleText(BuildContext context) {
    switch (purpose) {
      case ApprovalPurpose.issuance:
        return context.l10n.organizationApprovePageReceiveFromTitle(organization.legalName.l10nValue(context));
      case ApprovalPurpose.disclosure:
        return context.l10n.organizationApprovePageShareWithTitle(organization.legalName.l10nValue(context));
      case ApprovalPurpose.sign:
        return context.l10n.organizationApprovePageSignWithTitle(organization.legalName.l10nValue(context));
    }
  }

  Widget _buildInfoRows(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        isFirstInteractionWithOrganization
            ? IconRow(
                icon: Image.asset(WalletAssets.icon_first_share),
                text: Text(context.l10n.organizationApprovePageFirstInteraction),
              )
            : const SizedBox.shrink(),
        IconRow(
          icon: Icon(
            Icons.flag_outlined,
            color: context.colorScheme.primary,
          ),
          text: Text(
            context.l10n.organizationApprovePagePurpose(requestPurpose?.l10nValue(context) ?? purpose.name),
          ),
        ),
      ],
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
}

enum ApprovalPurpose { issuance, disclosure, sign }
