import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/data_attribute.dart';
import '../../common/widget/confirm_buttons.dart';
import '../../common/widget/data_attribute_row.dart';
import '../../common/widget/link_button.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../verification/model/organization.dart';

class IssuanceProofIdentityPage extends StatelessWidget {
  final VoidCallback onDecline;
  final VoidCallback onAccept;
  final Organization organization;
  final List<DataAttribute> attributes;

  const IssuanceProofIdentityPage({
    required this.onDecline,
    required this.onAccept,
    required this.organization,
    required this.attributes,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return CustomScrollView(
      restorationId: 'proof_identity_scrollview',
      slivers: <Widget>[
        const SliverToBoxAdapter(child: SizedBox(height: 32)),
        SliverToBoxAdapter(child: _buildHeaderSection(context)),
        const SliverToBoxAdapter(child: SizedBox(height: 8)),
        const SliverToBoxAdapter(child: Divider(height: 32)),
        SliverList(delegate: _getDataAttributesDelegate()),
        SliverToBoxAdapter(child: _buildDescriptionSection(context)),
        const SliverToBoxAdapter(child: Divider(height: 20)),
        SliverFillRemaining(
          hasScrollBody: false,
          fillOverscroll: true,
          child: ConfirmButtons(
            onAccept: onAccept,
            acceptText: AppLocalizations.of(context).issuanceProofIdentityPagePositiveCta,
            onDecline: onDecline,
            declineText: AppLocalizations.of(context).issuanceProofIdentityPageNegativeCta,
            acceptIcon: Icons.arrow_forward,
          ),
        ),
      ],
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            locale.issuanceProofIdentityPageTitle,
            style: Theme.of(context).textTheme.headline2,
          ),
          const SizedBox(height: 8),
          Text(
            locale.issuanceProofIdentityPageSubtitle(organization.name),
            style: Theme.of(context).textTheme.bodyText1,
          ),
        ],
      ),
    );
  }

  SliverChildBuilderDelegate _getDataAttributesDelegate() {
    return SliverChildBuilderDelegate(
      (context, index) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: DataAttributeRow(attribute: attributes[index]),
      ),
      childCount: attributes.length,
    );
  }

  Widget _buildDescriptionSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(left: 8.0),
      child: Align(
        alignment: AlignmentDirectional.centerStart,
        child: LinkButton(
          onPressed: () =>
              PlaceholderScreen.show(context, AppLocalizations.of(context).issuanceProofIdentityPageIncorrectCta),
          child: Text(AppLocalizations.of(context).issuanceProofIdentityPageIncorrectCta),
        ),
      ),
    );
  }
}
