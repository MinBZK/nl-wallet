import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/app_image.dart';
import '../../common/widget/attribute/data_attribute_row.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/secondary_button.dart';
import '../../common/widget/policy/policy_section.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../common/widget/wallet_scrollbar.dart';

class ConfirmAgreementPage extends StatelessWidget {
  final VoidCallback onDeclinePressed;
  final VoidCallback onAcceptPressed;
  final Policy policy;
  final Organization relyingParty;
  final Organization trustProvider;
  final List<DataAttribute> requestedAttributes;

  const ConfirmAgreementPage({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    required this.policy,
    required this.relyingParty,
    required this.trustProvider,
    required this.requestedAttributes,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return WalletScrollbar(
      child: CustomScrollView(
        slivers: <Widget>[
          const SliverSizedBox(height: 8),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          SliverList(delegate: _getDataAttributesDelegate()),
          const SliverSizedBox(height: 16),
          SliverToBoxAdapter(child: _buildDataIncorrectButton(context)),
          const SliverSizedBox(height: 16),
          SliverToBoxAdapter(child: PolicySection(relyingParty: relyingParty, policy: policy, addSignatureRow: true)),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverToBoxAdapter(child: _buildTrustProvider(context)),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: Container(
              alignment: Alignment.bottomCenter,
              child: ConfirmButtons(
                primaryButton: PrimaryButton(
                  key: const Key('acceptButton'),
                  onPressed: onAcceptPressed,
                  text: Text.rich(context.l10n.confirmAgreementPageConfirmCta.toTextSpan(context)),
                  icon: null,
                ),
                secondaryButton: SecondaryButton(
                  key: const Key('rejectButton'),
                  onPressed: onDeclinePressed,
                  icon: const Icon(Icons.block_flipped),
                  text: Text.rich(context.l10n.confirmAgreementPageCancelCta.toTextSpan(context)),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Image.asset(
            WalletAssets.illustration_sign_2,
            fit: BoxFit.cover,
            width: double.infinity,
          ),
          const SizedBox(height: 32),
          Text(
            context.l10n.confirmAgreementPageTitle,
            style: context.textTheme.displayMedium,
            textAlign: TextAlign.start,
          ),
        ],
      ),
    );
  }

  SliverChildBuilderDelegate _getDataAttributesDelegate() {
    return SliverChildBuilderDelegate(
      (context, index) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: DataAttributeRow(attribute: requestedAttributes[index]),
      ),
      childCount: requestedAttributes.length,
    );
  }

  Widget _buildDataIncorrectButton(BuildContext context) {
    return ListButton(
      onPressed: () => PlaceholderScreen.showGeneric(context),
      text: Text.rich(context.l10n.confirmAgreementPageDataIncorrectCta.toTextSpan(context)),
    );
  }

  Widget _buildTrustProvider(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Row(
        children: [
          AppImage(asset: trustProvider.logo),
          const SizedBox(width: 16),
          Expanded(
            child: Text(
              context.l10n.confirmAgreementPageSignProvider(trustProvider.displayName.l10nValue(context)),
              style: context.textTheme.bodyLarge,
            ),
          ),
        ],
      ),
    );
  }
}
