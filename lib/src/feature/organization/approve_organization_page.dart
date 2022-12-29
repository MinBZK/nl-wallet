import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../common/widget/confirm_buttons.dart';
import '../common/widget/placeholder_screen.dart';
import '../common/widget/sliver_sized_box.dart';
import '../common/widget/text_icon_button.dart';
import '../verification/model/organization.dart';

class ApproveOrganizationPage extends StatelessWidget {
  final VoidCallback onDecline;
  final VoidCallback onAccept;
  final Organization organization;
  final ApprovalPurpose purpose;

  const ApproveOrganizationPage({
    required this.onDecline,
    required this.onAccept,
    required this.organization,
    required this.purpose,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Scrollbar(
      thumbVisibility: true,
      child: CustomScrollView(
        restorationId: 'approve_organization_scrollview',
        slivers: <Widget>[
          const SliverSizedBox(height: 32),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          const SliverToBoxAdapter(child: Divider(height: 48)),
          SliverToBoxAdapter(child: _buildDescriptionSection(context)),
          const SliverToBoxAdapter(child: Divider(height: 48)),
          SliverToBoxAdapter(child: _buildDataIncorrectButton(context)),
          const SliverToBoxAdapter(child: Divider(height: 48)),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: Container(
              alignment: Alignment.bottomCenter,
              child: ConfirmButtons(
                onAccept: onAccept,
                acceptText: locale.approveOrganizationPageApproveCta,
                onDecline: onDecline,
                declineText: locale.approveOrganizationPageDenyCta,
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
            style: Theme.of(context).textTheme.headline2,
            textAlign: TextAlign.center,
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

  Widget _buildDescriptionSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            organization.shortName,
            style: Theme.of(context).textTheme.subtitle1,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 8),
          Text(
            organization.description,
            style: Theme.of(context).textTheme.bodyText1,
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }

  Widget _buildDataIncorrectButton(BuildContext context) {
    return Center(
      child: TextIconButton(
        child: Text(AppLocalizations.of(context).approveOrganizationPageIncorrectCta),
        onPressed: () => PlaceholderScreen.show(context),
      ),
    );
  }
}

enum ApprovalPurpose { issuance, verification, sign }
