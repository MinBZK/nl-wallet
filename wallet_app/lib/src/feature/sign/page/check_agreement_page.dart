import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/document.dart';
import '../../../domain/model/organization.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_assets.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/app_image.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/secondary_button.dart';
import '../../common/widget/document_section.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_scrollbar.dart';

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
    return WalletScrollbar(
      child: CustomScrollView(
        slivers: <Widget>[
          const SliverSizedBox(height: 8),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          const SliverToBoxAdapter(child: Divider()),
          SliverToBoxAdapter(
            child: DocumentSection(
              document: document,
              organization: organization,
              padding: const EdgeInsets.fromLTRB(16, 24, 16, 0),
            ),
          ),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverToBoxAdapter(child: _buildTrustProvider(context)),
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
                  onPressed: onAccept,
                  text: Text.rich(context.l10n.checkAgreementPageConfirmCta.toTextSpan(context)),
                  icon: null,
                ),
                secondaryButton: SecondaryButton(
                  key: const Key('rejectButton'),
                  onPressed: onDecline,
                  icon: const Icon(Icons.block_flipped),
                  text: Text.rich(context.l10n.checkAgreementPageCancelCta.toTextSpan(context)),
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
          TitleText(context.l10n.checkAgreementPageTitle),
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
    return ListButton(
      onPressed: () => PlaceholderScreen.showGeneric(context),
      text: Text.rich(context.l10n.checkAgreementPageDataIncorrectCta.toTextSpan(context)),
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
          ),
        ],
      ),
    );
  }
}
