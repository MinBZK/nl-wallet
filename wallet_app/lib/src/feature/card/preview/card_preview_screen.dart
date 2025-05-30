import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../navigation/secured_page_route.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/widget/attribute/data_attribute_row.dart';
import '../../common/widget/button/bottom_back_button.dart';
import '../../common/widget/button/icon/help_icon_button.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/card/wallet_card_item.dart';
import '../../common/widget/fade_in_at_offset.dart';
import '../../common/widget/spacer/sliver_divider.dart';
import '../../common/widget/spacer/sliver_sized_box.dart';
import '../../common/widget/text/body_text.dart';
import '../../common/widget/text/title_text.dart';
import '../../common/widget/wallet_app_bar.dart';
import '../../common/widget/wallet_scrollbar.dart';
import '../data/card_data_incorrect_screen.dart';

class CardPreviewScreen extends StatelessWidget {
  final WalletCard card;

  const CardPreviewScreen({required this.card, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: FadeInAtOffset(
          visibleOffset: 110,
          appearOffset: 90,
          child: Text(card.title.l10nValue(context)),
        ),
        actions: const [HelpIconButton()],
      ),
      body: SafeArea(
        child: _buildBody(context),
      ),
    );
  }

  Widget _buildBody(BuildContext context) {
    return Column(
      children: [
        Expanded(
          child: WalletScrollbar(
            child: CustomScrollView(
              slivers: [
                const SliverSizedBox(height: 12),
                _buildHeader(context),
                const SliverDivider(),
                _buildDataAttributes(context, card.attributes),
              ],
            ),
          ),
        ),
        const BottomBackButton(),
      ],
    );
  }

  Widget _buildDataAttributes(BuildContext context, List<DataAttribute> attributes) {
    final List<Widget> slivers = [];

    // Data attributes
    slivers.add(const SliverSizedBox(height: 16));
    for (final attribute in attributes) {
      slivers.add(
        SliverToBoxAdapter(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: DataAttributeRow(attribute: attribute),
          ),
        ),
      );
    }

    // Incorrect button
    slivers.add(const SliverSizedBox(height: 16));
    slivers.add(SliverToBoxAdapter(child: _buildIncorrectButton(context)));
    slivers.add(const SliverSizedBox(height: 24));

    return SliverMainAxisGroup(slivers: slivers);
  }

  Widget _buildIncorrectButton(BuildContext context) {
    return ListButton(
      text: Text.rich(context.l10n.cardPreviewScreenIncorrectCta.toTextSpan(context)),
      onPressed: () => CardDataIncorrectScreen.show(context),
    );
  }

  Widget _buildHeader(BuildContext context) {
    return SliverToBoxAdapter(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            SizedBox(
              width: 110,
              child: WalletCardItem.fromWalletCard(context, card, showText: false),
            ),
            const SizedBox(height: 8),
            TitleText(card.title.l10nValue(context)),
            BodyText(context.l10n.cardPreviewScreenIssuedBy(card.issuer.displayName.l10nValue(context))),
            const SizedBox(height: 24),
          ],
        ),
      ),
    );
  }

  static Future<void> show(BuildContext context, {required WalletCard card}) {
    return Navigator.push(
      context,
      SecuredPageRoute(builder: (c) => CardPreviewScreen(card: card)),
    );
  }
}
