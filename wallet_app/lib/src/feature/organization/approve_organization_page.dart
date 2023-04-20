import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../common/widget/button/confirm_buttons.dart';
import '../common/widget/icon_row.dart';
import '../common/widget/placeholder_screen.dart';
import '../common/widget/sliver_sized_box.dart';
import '../verification/model/organization.dart';
import 'widget/organization_row.dart';

class ApproveOrganizationPage extends StatelessWidget {
  /// Callback that is triggered when the user approves the request
  final VoidCallback onAccept;

  /// Callback that is triggered when the user declines the request
  final VoidCallback onDecline;

  /// The organization that user is interacting with
  final Organization organization;

  /// Tells the Page in which flow it's currently used, used to select the correct string resources
  final ApprovalPurpose purpose;

  /// If true, the 'first interaction' banner will be shown.
  final bool isFirstInteractionWithOrganization;

  const ApproveOrganizationPage({
    required this.onDecline,
    required this.onAccept,
    required this.organization,
    required this.purpose,
    this.isFirstInteractionWithOrganization = true,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      thumbVisibility: true,
      child: CustomScrollView(
        controller: ScrollController(),
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
              onTap: () => PlaceholderScreen.show(context),
              organizationName: organization.shortName,
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
                onAccept: onAccept,
                acceptIcon: Icons.arrow_forward,
                acceptText: _approveButtonText(context),
                onDecline: onDecline,
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
          SizedBox(
            width: 64,
            height: 64,
            child: ClipRRect(
              borderRadius: BorderRadius.circular(6.4),
              child: Image.asset(organization.logoUrl),
            ),
          ),
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
        return locale.approveOrganizationPageReceiveFromTitle(organization.shortName);
      case ApprovalPurpose.verification:
        return locale.approveOrganizationPageShareWithTitle(organization.shortName);
      case ApprovalPurpose.sign:
        return locale.approveOrganizationPageSignWithTitle(organization.shortName);
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
                text: Text(locale.approveOrganizationPageFirstInteraction),
              )
            : const SizedBox.shrink(),
        IconRow(
          icon: Icon(
            Icons.flag_outlined,
            color: Theme.of(context).colorScheme.primary,
          ),
          text: Text(
            //TODO: Replace [purpose.name] with the actual purpose once the mocks are updated.
            locale.approveOrganizationPagePurpose(purpose.name),
          ),
        ),
      ],
    );
  }

  String _approveButtonText(BuildContext context) {
    final locale = AppLocalizations.of(context);
    switch (purpose) {
      case ApprovalPurpose.issuance:
        return locale.approveOrganizationPageApproveCta;
      case ApprovalPurpose.verification:
        return locale.approveOrganizationPageShareWithApproveCta;
      case ApprovalPurpose.sign:
        return locale.approveOrganizationPageApproveCta;
    }
  }

  String _declineButtonText(BuildContext context) {
    final locale = AppLocalizations.of(context);
    switch (purpose) {
      case ApprovalPurpose.issuance:
        return locale.approveOrganizationPageDenyCta;
      case ApprovalPurpose.verification:
        return locale.approveOrganizationPageShareWithDenyCta;
      case ApprovalPurpose.sign:
        return locale.approveOrganizationPageDenyCta;
    }
  }
}

enum ApprovalPurpose { issuance, verification, sign }
