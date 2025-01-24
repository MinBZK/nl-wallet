import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/flow_progress.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../common/widget/attribute/attribute_row.dart';
import '../../../common/widget/button/confirm/confirm_buttons.dart';
import '../../../common/widget/button/primary_button.dart';
import '../../../common/widget/button/secondary_button.dart';
import '../../../common/widget/sliver_sized_box.dart';
import '../../../common/widget/sliver_wallet_app_bar.dart';
import '../../../common/widget/text/body_text.dart';
import '../../../common/widget/wallet_scrollbar.dart';
import '../wallet_personalize_data_incorrect_screen.dart';

class WalletPersonalizeCheckDataOfferingPage extends StatelessWidget {
  final VoidCallback onAcceptPressed;
  final VoidCallback onRejectPressed;
  final List<Attribute> attributes;
  final FlowProgress progress;

  const WalletPersonalizeCheckDataOfferingPage({
    required this.onAcceptPressed,
    required this.onRejectPressed,
    required this.attributes,
    required this.progress,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: WalletScrollbar(
        child: CustomScrollView(
          slivers: <Widget>[
            SliverWalletAppBar(
              title: context.l10n.walletPersonalizeCheckDataOfferingPageTitle,
              scrollController: PrimaryScrollController.maybeOf(context),
              progress: progress,
            ),
            SliverToBoxAdapter(child: _buildHeaderSection(context)),
            const SliverSizedBox(height: 24),
            const SliverToBoxAdapter(child: Divider()),
            const SliverSizedBox(height: 12),
            SliverList(delegate: _getDataAttributesDelegate()),
            const SliverSizedBox(height: 12),
            SliverFillRemaining(
              hasScrollBody: false,
              fillOverscroll: true,
              child: Column(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  const Divider(),
                  _buildBottomSection(context),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 0),
      alignment: Alignment.centerLeft,
      child: BodyText(context.l10n.walletPersonalizeCheckDataOfferingPageSubtitle),
    );
  }

  SliverChildBuilderDelegate _getDataAttributesDelegate() {
    return SliverChildBuilderDelegate(
      (context, index) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        child: AttributeRow(attribute: attributes[index]),
      ),
      childCount: attributes.length,
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return ConfirmButtons(
      primaryButton: PrimaryButton(
        key: const Key('acceptButton'),
        onPressed: onAcceptPressed,
        icon: const Icon(Icons.check_rounded),
        text: Text.rich(context.l10n.walletPersonalizeCheckDataOfferingPageAcceptCta.toTextSpan(context)),
      ),
      secondaryButton: SecondaryButton(
        key: const Key('rejectButton'),
        onPressed: () => WalletPersonalizeDataIncorrectScreen.show(context, onRejectPressed),
        text: Text(
          context.l10n.walletPersonalizeCheckDataOfferingPageDeclineCta,
          semanticsLabel: context.l10n.walletPersonalizeCheckDataOfferingPageDeclineCtaSemanticsLabel,
        ),
        icon: const Icon(Icons.block_flipped),
      ),
    );
  }
}
