import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/ui_attribute.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../common/widget/attribute/attribute_row.dart';
import '../../../common/widget/button/confirm_buttons.dart';
import '../../../common/widget/sliver_sized_box.dart';
import '../wallet_personalize_data_incorrect_screen.dart';

class WalletPersonalizeCheckDataOfferingPage extends StatelessWidget {
  final VoidCallback onAcceptPressed;
  final VoidCallback onRejectPressed;
  final List<UiAttribute> attributes;

  const WalletPersonalizeCheckDataOfferingPage({
    required this.onAcceptPressed,
    required this.onRejectPressed,
    required this.attributes,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      child: CustomScrollView(
        slivers: <Widget>[
          const SliverSizedBox(height: 32),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          const SliverSizedBox(height: 32),
          const SliverToBoxAdapter(child: Divider(height: 1)),
          const SliverSizedBox(height: 12),
          SliverList(delegate: _getDataAttributesDelegate()),
          const SliverSizedBox(height: 16),
          const SliverToBoxAdapter(child: Divider(height: 24)),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: Align(
              alignment: Alignment.bottomCenter,
              child: _buildBottomSection(context),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 0),
      alignment: Alignment.centerLeft,
      child: Text(
        context.l10n.walletPersonalizeCheckDataOfferingPageTitle,
        style: context.textTheme.displayMedium,
      ),
    );
  }

  SliverChildBuilderDelegate _getDataAttributesDelegate() {
    return SliverChildBuilderDelegate(
      (context, index) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 12),
        child: AttributeRow(attribute: attributes[index]),
      ),
      childCount: attributes.length,
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return ConfirmButtons(
      onDeclinePressed: () => WalletPersonalizeDataIncorrectScreen.show(context, onRejectPressed),
      onAcceptPressed: onAcceptPressed,
      acceptText: context.l10n.walletPersonalizeCheckDataOfferingPageAcceptCta,
      declineText: context.l10n.walletPersonalizeCheckDataOfferingPageDeclineCta,
      declineTextSemanticsLabel: context.l10n.walletPersonalizeCheckDataOfferingPageDeclineCtaSemanticsLabel,
    );
  }
}
