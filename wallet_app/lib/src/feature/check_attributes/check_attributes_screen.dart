import 'package:flutter/material.dart';

import '../../domain/model/attribute/attribute.dart';
import '../../domain/model/attribute/data_attribute.dart';
import '../../domain/model/wallet_card.dart';
import '../../util/extension/build_context_extension.dart';
import '../common/widget/attribute/data_attribute_section.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/link_button.dart';
import '../common/widget/sliver_sized_box.dart';

class CheckAttributesScreen extends StatelessWidget {
  final Map<WalletCard, List<DataAttribute>> cardsToAttributes;
  final VoidCallback? onDataIncorrectPressed;

  const CheckAttributesScreen({
    required this.cardsToAttributes,
    this.onDataIncorrectPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(context.l10n.checkAttributesScreenTitle),
      ),
      body: SafeArea(
        child: Column(
          children: [
            Expanded(
              child: _buildContent(context),
            ),
            const BottomBackButton(showDivider: true),
          ],
        ),
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    return Scrollbar(
      child: CustomScrollView(
        slivers: [
          ..._generateDataSectionSlivers(context),
          SliverToBoxAdapter(child: _buildDataIncorrectButton(context)),
          const SliverSizedBox(height: 32),
        ],
      ),
    );
  }

  List<Widget> _generateDataSectionSlivers(BuildContext context) {
    final dataSections = cardsToAttributes.entries.map(
      (cardToAttributes) => Column(
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 24),
            child: DataAttributeSection(
              sourceCardTitle: cardToAttributes.key.front.title.l10nValue(context),
              attributes: cardToAttributes.value,
            ),
          ),
          const Divider(height: 1),
        ],
      ),
    );
    return dataSections.map((e) => SliverToBoxAdapter(child: e)).toList();
  }

  Widget _buildDataIncorrectButton(BuildContext context) {
    if (onDataIncorrectPressed == null) return const SizedBox.shrink();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        LinkButton(
          onPressed: () => onDataIncorrectPressed!(),
          customPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
          child: Text(context.l10n.checkAttributesScreenDataIncorrectCta),
        ),
        const Divider(height: 1),
      ],
    );
  }

  static void show(
    BuildContext context,
    Map<WalletCard, List<DataAttribute>> cardsToAttributes, {
    VoidCallback? onDataIncorrectPressed,
  }) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (c) => CheckAttributesScreen(
          cardsToAttributes: cardsToAttributes,
          onDataIncorrectPressed: onDataIncorrectPressed,
        ),
      ),
    );
  }
}
