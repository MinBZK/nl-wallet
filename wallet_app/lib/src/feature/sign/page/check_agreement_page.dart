import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/document.dart';
import '../../../domain/model/organization.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/app_image.dart';
import '../../common/widget/button/confirm/confirm_button.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/link_button.dart';
import '../../common/widget/document_section.dart';
import '../../common/widget/sliver_sized_box.dart';

class CheckAgreementPage extends StatelessWidget {
  final VoidCallback onDecline;
  final VoidCallback onAccept;
  final Organization organization;
  final Organization trustProvider;
  final Document document;

  const CheckAgreementPage({
    required this.onDecline,
    required this.onAccept,
    required this.organization,
    required this.trustProvider,
    required this.document,
    super.key,
  });

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
              document: document,
              organization: organization,
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
                primaryButton: ConfirmButton.accept(
                  onPressed: onAccept,
                  text: context.l10n.checkAgreementPageConfirmCta,
                ),
                secondaryButton: ConfirmButton.reject(
                  onPressed: onDecline,
                  icon: Icons.block_flipped,
                  text: context.l10n.checkAgreementPageCancelCta,
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
          AppImage(asset: trustProvider.logo),
          const SizedBox(width: 16),
          Expanded(
            child: Text(
              context.l10n.checkAgreementPageSignProvider(
                organization.displayName.l10nValue(context),
                trustProvider.displayName.l10nValue(context),
              ),
              style: context.textTheme.bodyLarge,
            ),
          )
        ],
      ),
    );
  }
}
