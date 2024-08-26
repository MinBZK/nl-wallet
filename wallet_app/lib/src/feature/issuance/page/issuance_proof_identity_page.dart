import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/attribute/attribute_row.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/secondary_button.dart';
import '../../common/widget/policy/policy_section.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../common/widget/wallet_scrollbar.dart';

class IssuanceProofIdentityPage extends StatelessWidget {
  final VoidCallback onDeclinePressed;
  final VoidCallback onAcceptPressed;
  final Organization organization;
  final List<Attribute> attributes;
  final Policy policy;
  final bool isRefreshFlow;

  const IssuanceProofIdentityPage({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    required this.organization,
    required this.attributes,
    required this.policy,
    required this.isRefreshFlow,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return WalletScrollbar(
      child: CustomScrollView(
        restorationId: 'proof_identity_scrollview',
        slivers: <Widget>[
          const SliverSizedBox(height: 32),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          const SliverSizedBox(height: 8),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverList(delegate: _getDataAttributesDelegate()),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverToBoxAdapter(child: PolicySection(policy)),
          const SliverSizedBox(height: 16),
          SliverToBoxAdapter(child: _buildDataIncorrectButton(context)),
          const SliverSizedBox(height: 24),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: Container(
              alignment: Alignment.bottomCenter,
              child: ConfirmButtons(
                primaryButton: PrimaryButton(
                  key: const Key('acceptButton'),
                  onPressed: onAcceptPressed,
                  text: Text.rich(context.l10n.issuanceProofIdentityPagePositiveCta.toTextSpan(context)),
                ),
                secondaryButton: SecondaryButton(
                  key: const Key('rejectButton'),
                  icon: const Icon(Icons.block_flipped),
                  onPressed: onDeclinePressed,
                  text: Text.rich(context.l10n.issuanceProofIdentityPageNegativeCta.toTextSpan(context)),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    final issuanceProofIdentityPageSubtitle = isRefreshFlow
        ? context.l10n.issuanceProofIdentityPageRefreshDataSubtitle(organization.displayName.l10nValue(context))
        : context.l10n.issuanceProofIdentityPageSubtitle(organization.displayName.l10nValue(context));

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
    return SliverChildBuilderDelegate(
      (context, index) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: AttributeRow(attribute: attributes[index]),
      ),
      childCount: attributes.length,
    );
  }

  Widget _buildDataIncorrectButton(BuildContext context) {
    return ListButton(
      text: Text.rich(context.l10n.issuanceProofIdentityPageIncorrectCta.toTextSpan(context)),
      onPressed: () => PlaceholderScreen.showGeneric(context),
    );
  }
}
