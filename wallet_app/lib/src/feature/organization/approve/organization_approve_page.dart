import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/button/confirm_buttons.dart';
import '../../common/widget/icon_row.dart';
import '../../common/widget/organization/organization_logo.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../verification/model/organization.dart';
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
  final String? requestPurpose;

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
      thumbVisibility: true,
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
              onTap: () => OrganizationDetailScreen.show(
                context,
                organization.id,
                onReportIssuePressed: onReportIssuePressed,
              ),
              organizationName: organization.category,
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
          OrganizationLogo(image: AssetImage(organization.logoUrl), size: 64),
          const SizedBox(height: 24),
          Text(
            _headerTitleText(context),
            style: Theme.of(context).textTheme.displayMedium,
            textAlign: TextAlign.start,
          ),
        ],
      ),
    );
  }

  String _headerTitleText(BuildContext context) {
    final locale = AppLocalizations.of(context);
    switch (purpose) {
      case ApprovalPurpose.issuance:
        return locale.organizationApprovePageReceiveFromTitle(organization.name);
      case ApprovalPurpose.verification:
        return locale.organizationApprovePageShareWithTitle(organization.name);
      case ApprovalPurpose.sign:
        return locale.organizationApprovePageSignWithTitle(organization.name);
    }
  }

  Widget _buildInfoRows(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        isFirstInteractionWithOrganization
            ? IconRow(
                icon: Image.asset('assets/images/ic_first_share.png'),
                text: Text(locale.organizationApprovePageFirstInteraction),
              )
            : const SizedBox.shrink(),
        IconRow(
          icon: Icon(
            Icons.flag_outlined,
            color: Theme.of(context).colorScheme.primary,
          ),
          text: Text(
            locale.organizationApprovePagePurpose(requestPurpose ?? purpose.name),
          ),
        ),
      ],
    );
  }

  String _approveButtonText(BuildContext context) {
    final locale = AppLocalizations.of(context);
    switch (purpose) {
      case ApprovalPurpose.issuance:
        return locale.organizationApprovePageApproveCta;
      case ApprovalPurpose.verification:
        return locale.organizationApprovePageShareWithApproveCta;
      case ApprovalPurpose.sign:
        return locale.organizationApprovePageApproveCta;
    }
  }

  String _declineButtonText(BuildContext context) {
    final locale = AppLocalizations.of(context);
    switch (purpose) {
      case ApprovalPurpose.issuance:
        return locale.organizationApprovePageDenyCta;
      case ApprovalPurpose.verification:
        return locale.organizationApprovePageShareWithDenyCta;
      case ApprovalPurpose.sign:
        return locale.organizationApprovePageDenyCta;
    }
  }
}

enum ApprovalPurpose { issuance, verification, sign }
