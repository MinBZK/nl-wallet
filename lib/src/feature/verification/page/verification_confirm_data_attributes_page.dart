import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../wallet_routes.dart';
import '../../common/widget/confirm_buttons.dart';
import '../../common/widget/data_attribute_row.dart';
import '../../common/widget/link_button.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../model/verification_flow.dart';
import '../widget/policy_row.dart';

class VerificationConfirmDataAttributesPage extends StatelessWidget {
  final VoidCallback onDecline;
  final VoidCallback onAccept;
  final VerificationFlow flow;

  const VerificationConfirmDataAttributesPage({
    required this.onDecline,
    required this.onAccept,
    required this.flow,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return CustomScrollView(
      restorationId: 'confirm_data_attributes_scrollview',
      slivers: <Widget>[
        const SliverSizedBox(height: 8),
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
          child: Container(
            alignment: Alignment.bottomCenter,
            child: ConfirmButtons(
              onAccept: onAccept,
              acceptText: locale.verificationConfirmDataAttributesPageApproveCta,
              onDecline: onDecline,
              declineText: locale.verificationConfirmDataAttributesPageDenyCta,
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Text(
        AppLocalizations.of(context).verificationConfirmDataAttributesPageShareDataTitle,
        style: Theme.of(context).textTheme.headline2,
        textAlign: TextAlign.start,
      ),
    );
  }

  SliverChildBuilderDelegate _getDataAttributesDelegate() {
    return SliverChildBuilderDelegate(
      (context, index) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: DataAttributeRow(attribute: flow.requestedDataAttributes[index]),
      ),
      childCount: flow.requestedDataAttributes.length,
    );
  }

  Widget _buildDataIncorrectButton(BuildContext context) {
    return Align(
      alignment: AlignmentDirectional.centerStart,
      child: LinkButton(
        onPressed: () => PlaceholderScreen.show(context, 'Klopt er iets niet?'),
        child: Padding(
          padding: const EdgeInsets.only(left: 8.0),
          child: Text(AppLocalizations.of(context).verificationConfirmDataAttributesPageIncorrectCta),
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
            text: locale.verificationConfirmDataAttributesPageDataRetentionDuration(flow.policy.storageDuration.inDays),
          ),
          PolicyRow(
            icon: Icons.share,
            text: flow.policy.dataIsShared
                ? locale.verificationConfirmDataAttributesPageDataWillBeShared
                : locale.verificationConfirmDataAttributesPageDataWillNotBeShared,
          ),
          PolicyRow(
            icon: Icons.delete_outline,
            text: flow.policy.deletionCanBeRequested
                ? locale.verificationConfirmDataAttributesPageDataCanBeDeleted
                : locale.verificationConfirmDataAttributesPageDataCanNotBeDeleted,
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
          arguments: flow.id.toString(),
        ),
        child: Padding(
          padding: const EdgeInsets.only(left: 8.0),
          child: Text(AppLocalizations.of(context).verificationConfirmDataAttributesPageAllTermsCta),
        ),
      ),
    );
  }
}
