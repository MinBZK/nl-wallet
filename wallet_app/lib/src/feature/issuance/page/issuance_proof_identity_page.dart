import 'package:flutter/material.dart';

import '../../../domain/model/issuance_flow.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/attribute/attribute_row.dart';
import '../../common/widget/button/confirm_buttons.dart';
import '../../common/widget/button/link_button.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../common/widget/policy/policy_section.dart';
import '../../common/widget/sliver_sized_box.dart';

class IssuanceProofIdentityPage extends StatelessWidget {
  final VoidCallback onDeclinePressed;
  final VoidCallback onAcceptPressed;
  final IssuanceFlow flow;
  final bool isRefreshFlow;

  const IssuanceProofIdentityPage({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    required this.flow,
    required this.isRefreshFlow,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      child: CustomScrollView(
        restorationId: 'proof_identity_scrollview',
        slivers: <Widget>[
          const SliverSizedBox(height: 32),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          const SliverSizedBox(height: 8),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverList(delegate: _getDataAttributesDelegate()),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverToBoxAdapter(child: PolicySection(flow.policy)),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverToBoxAdapter(child: _buildDataIncorrectButton(context)),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: Container(
              alignment: Alignment.bottomCenter,
              child: ConfirmButtons(
                onAcceptPressed: onAcceptPressed,
                acceptText: context.l10n.issuanceProofIdentityPagePositiveCta,
                onDeclinePressed: onDeclinePressed,
                declineText: context.l10n.issuanceProofIdentityPageNegativeCta,
                acceptIcon: Icons.arrow_forward,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    final organization = flow.organization;
    final issuanceProofIdentityPageSubtitle = isRefreshFlow
        ? context.l10n.issuanceProofIdentityPageRefreshDataSubtitle(organization.shortName)
        : context.l10n.issuanceProofIdentityPageSubtitle(organization.shortName);

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: MergeSemantics(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              context.l10n.issuanceProofIdentityPageTitle,
              style: context.textTheme.displayMedium,
            ),
            const SizedBox(height: 8),
            Text(
              issuanceProofIdentityPageSubtitle,
              style: context.textTheme.bodyLarge,
            ),
          ],
        ),
      ),
    );
  }

  SliverChildBuilderDelegate _getDataAttributesDelegate() {
    final attributes = flow.attributes;
    return SliverChildBuilderDelegate(
      (context, index) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: AttributeRow(attribute: attributes[index]),
      ),
      childCount: attributes.length,
    );
  }

  Widget _buildDataIncorrectButton(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(left: 8),
      child: Align(
        alignment: AlignmentDirectional.centerStart,
        child: LinkButton(
          onPressed: () => PlaceholderScreen.show(context),
          child: Text(context.l10n.issuanceProofIdentityPageIncorrectCta),
        ),
      ),
    );
  }
}
