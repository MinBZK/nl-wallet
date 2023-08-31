import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/widget/button/confirm_buttons.dart';
import '../../common/widget/button/link_button.dart';
import '../../common/widget/document_section.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../model/sign_flow.dart';

class CheckAgreementPage extends StatelessWidget {
  final VoidCallback onDecline;
  final VoidCallback onAccept;
  final SignFlow flow;

  const CheckAgreementPage({
    required this.onDecline,
    required this.onAccept,
    required this.flow,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      child: CustomScrollView(
        slivers: <Widget>[
          const SliverSizedBox(height: 8),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          const SliverToBoxAdapter(child: Divider(height: 1)),
          SliverToBoxAdapter(
            child: DocumentSection(
              document: flow.document,
              organization: flow.organization,
              padding: const EdgeInsets.fromLTRB(16, 24, 16, 0),
            ),
          ),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverToBoxAdapter(child: _buildTrustProvider(context)),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverToBoxAdapter(child: _buildDataIncorrectButton(context)),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: Container(
              alignment: Alignment.bottomCenter,
              child: ConfirmButtons(
                onAcceptPressed: onAccept,
                acceptText: context.l10n.checkAgreementPageConfirmCta,
                onDeclinePressed: onDecline,
                declineText: context.l10n.checkAgreementPageCancelCta,
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
            WalletAssets.illustration_sign_1,
            fit: BoxFit.cover,
            width: double.infinity,
          ),
          const SizedBox(height: 32),
          Text(
            context.l10n.checkAgreementPageTitle,
            style: context.textTheme.displayMedium,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 8),
          Text(
            context.l10n.checkAgreementPageSubtitle,
            style: context.textTheme.bodyLarge,
            textAlign: TextAlign.start,
          ),
        ],
      ),
    );
  }

  Widget _buildDataIncorrectButton(BuildContext context) {
    return Align(
      alignment: AlignmentDirectional.centerStart,
      child: LinkButton(
        onPressed: () => PlaceholderScreen.show(context),
        child: Padding(
          padding: const EdgeInsets.only(left: 8),
          child: Text(context.l10n.checkAgreementPageDataIncorrectCta),
        ),
      ),
    );
  }

  Widget _buildTrustProvider(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Row(
        children: [
          Image.asset(flow.trustProvider.logoUrl),
          const SizedBox(width: 16),
          Expanded(
            child: Text(
              context.l10n.checkAgreementPageSignProvider(
                flow.organization.shortName,
                flow.trustProvider.name,
              ),
              style: context.textTheme.bodyLarge,
            ),
          )
        ],
      ),
    );
  }
}
