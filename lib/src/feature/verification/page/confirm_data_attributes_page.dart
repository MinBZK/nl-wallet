import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../wallet_routes.dart';
import '../../common/widget/data_attribute_row.dart';
import '../../common/widget/link_button.dart';
import '../model/verification_request.dart';
import '../widget/confirm_buttons.dart';
import '../widget/policy_row.dart';

class ConfirmDataAttributesPage extends StatelessWidget {
  final VoidCallback onDecline;
  final VoidCallback onAccept;
  final VerificationRequest request;

  const ConfirmDataAttributesPage({
    required this.onDecline,
    required this.onAccept,
    required this.request,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return CustomScrollView(
      restorationId: 'confirm_data_attributes_scrollview',
      slivers: <Widget>[
        const SliverToBoxAdapter(child: SizedBox(height: 8)),
        SliverToBoxAdapter(child: _buildHeaderSection(context)),
        SliverList(delegate: _getDataAttributesDelegate()),
        SliverToBoxAdapter(child: _buildDataIncorrectButton(context)),
        const SliverToBoxAdapter(child: Divider(height: 32)),
        SliverToBoxAdapter(child: _buildPolicySection(context)),
        SliverToBoxAdapter(child: _buildShowPoliciesButton(context)),
        const SliverToBoxAdapter(child: Divider(height: 32)),
        SliverFillRemaining(
          hasScrollBody: false,
          fillOverscroll: true,
          child: ConfirmButtons(
            onAccept: onAccept,
            acceptText: locale.verificationScreenApproveAttributesCta,
            onDecline: onDecline,
            declineText: locale.verificationScreenDenyAttributesCta,
          ),
        ),
      ],
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Text(
        AppLocalizations.of(context).verificationScreenShareDataTitle,
        style: Theme.of(context).textTheme.headline2,
        textAlign: TextAlign.start,
      ),
    );
  }

  SliverChildBuilderDelegate _getDataAttributesDelegate() {
    return SliverChildBuilderDelegate(
      (context, index) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: DataAttributeRow(attribute: request.attributes[index]),
      ),
      childCount: request.attributes.length,
    );
  }

  Widget _buildDataIncorrectButton(BuildContext context) {
    return Align(
      alignment: AlignmentDirectional.centerStart,
      child: LinkButton(
        onPressed: () {},
        child: Padding(
          padding: const EdgeInsets.only(left: 8.0),
          child: Text(AppLocalizations.of(context).verificationScreenIncorrectCta),
        ),
      ),
    );
  }

  Widget _buildPolicySection(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        children: [
          PolicyRow(
            icon: Icons.access_time_rounded,
            text: locale.verificationScreenDataRetentionDuration(request.policy.storageDuration.inDays),
          ),
          PolicyRow(
            icon: Icons.share,
            text: request.policy.dataIsShared
                ? locale.verificationScreenDataWillBeShared
                : locale.verificationScreenDataWillNotBeShared,
          ),
          PolicyRow(
            icon: Icons.delete_outline,
            text: request.policy.deletionCanBeRequested
                ? locale.verificationScreenDataCanBeDeleted
                : locale.verificationScreenDataCanNotBeDeleted,
          ),
        ],
      ),
    );
  }

  Widget _buildShowPoliciesButton(BuildContext context) {
    return Align(
      alignment: AlignmentDirectional.centerStart,
      child: LinkButton(
        onPressed: () => Navigator.restorablePushNamed(
          context,
          WalletRoutes.verifierPolicyRoute,
          arguments: request.id.toString(),
        ),
        child: Padding(
          padding: const EdgeInsets.only(left: 8.0),
          child: Text(AppLocalizations.of(context).verificationScreenAllTermsCta),
        ),
      ),
    );
  }
}
