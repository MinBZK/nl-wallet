import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../common/widget/confirm_buttons.dart';
import '../common/widget/placeholder_screen.dart';
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
    return CustomScrollView(
      restorationId: 'confirm_verifier_scrollview',
      slivers: <Widget>[
        const SliverToBoxAdapter(child: SizedBox(height: 32)),
        SliverToBoxAdapter(child: _buildHeaderSection(context)),
        const SliverToBoxAdapter(child: Divider(height: 48)),
        SliverToBoxAdapter(child: _buildDescriptionSection(context)),
        const SliverToBoxAdapter(child: Divider(height: 48)),
        SliverToBoxAdapter(child: _buildDataIncorrectButton(context)),
        SliverFillRemaining(
          hasScrollBody: false,
          fillOverscroll: true,
          child: ConfirmButtons(
            onAccept: onAccept,
            acceptText: locale.verificationScreenApproveOrganizationCta,
            onDecline: onDecline,
            declineText: locale.verificationScreenDenyOrganizationCta,
          ),
        ),
      ],
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    final String title;
    switch (purpose) {
      case ApprovalPurpose.issuance:
        title = AppLocalizations.of(context).verificationScreenReceiveFromTitle(organization.shortName);
        break;
      case ApprovalPurpose.verification:
        title = AppLocalizations.of(context).verificationScreenShareWithTitle(organization.shortName);
        break;
    }
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
              child: organization.logoUrl == null
                  ? Container(
                      color: Theme.of(context).colorScheme.secondaryContainer,
                      child: const Icon(Icons.question_mark),
                    )
                  : Image.asset(organization.logoUrl!),
            ),
          ),
          const SizedBox(height: 24),
          Text(
            title,
            style: Theme.of(context).textTheme.headline2,
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }

  Widget _buildDescriptionSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            organization.name,
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
        child: Text(AppLocalizations.of(context).verificationScreenIncorrectCta),
        onPressed: () => PlaceholderScreen.show(context, 'Klopt er iets niet?'),
      ),
    );
  }
}

enum ApprovalPurpose { issuance, verification }
