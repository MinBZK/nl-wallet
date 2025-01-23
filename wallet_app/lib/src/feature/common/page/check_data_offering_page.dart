import 'package:flutter/material.dart';

import '../../../domain/model/attribute/data_attribute.dart';
import '../../../domain/model/card_front.dart';
import '../../../util/extension/build_context_extension.dart';
import '../widget/attribute/data_attribute_row.dart';
import '../widget/button/list_button.dart';
import '../widget/card/wallet_card_item.dart';
import '../widget/sliver_sized_box.dart';
import '../widget/wallet_scrollbar.dart';

/// Generic Page that displays the attributes so the user can check them.
/// Consumer needs to provide the [bottomSection] to handle any user actions.
class CheckDataOfferingPage extends StatelessWidget {
  final List<DataAttribute> attributes;
  final Widget bottomSection;
  final String title;
  final String? overline, subtitle, footerCta;
  final CardFront? cardFront;
  final bool showHeaderAttributesDivider;

  const CheckDataOfferingPage({
    required this.title,
    this.overline,
    this.subtitle,
    this.cardFront,
    this.showHeaderAttributesDivider = true,
    this.footerCta,
    required this.bottomSection,
    required this.attributes,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return WalletScrollbar(
      child: CustomScrollView(
        restorationId: 'check_data_offering_scrollview',
        slivers: <Widget>[
          const SliverSizedBox(height: 32),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          SliverToBoxAdapter(child: _buildCardFront(context)),
          SliverSizedBox(height: showHeaderAttributesDivider ? 24 : 12),
          if (showHeaderAttributesDivider) const SliverToBoxAdapter(child: Divider()),
          const SliverSizedBox(height: 12),
          SliverList(delegate: _getDataAttributesDelegate()),
          const SliverSizedBox(height: 16),
          SliverToBoxAdapter(child: _buildFooterSection(context)),
          const SliverToBoxAdapter(child: Divider()),
          SliverFillRemaining(hasScrollBody: false, fillOverscroll: true, child: _buildBottomSection()),
        ],
      ),
    );
  }

  Widget _buildCardFront(BuildContext context) {
    final cardFront = this.cardFront;
    if (cardFront == null) return const SizedBox.shrink();
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 24, 16, 0),
      child: WalletCardItem.fromCardFront(context: context, front: cardFront),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    final overline = this.overline;
    final subtitle = this.subtitle;
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 0),
      child: MergeSemantics(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (overline != null)
              Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: Text(
                  overline,
                  style: context.textTheme.labelSmall?.copyWith(color: context.colorScheme.primary),
                ),
              ),
            Text(
              title,
              style: context.textTheme.displayMedium,
            ),
            if (subtitle != null)
              Padding(
                padding: const EdgeInsets.only(top: 8),
                child: Text(
                  subtitle,
                  style: context.textTheme.bodyLarge,
                ),
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
        child: DataAttributeRow(attribute: attributes[index]),
      ),
      childCount: attributes.length,
    );
  }

  Widget _buildFooterSection(BuildContext context) {
    final footerCta = this.footerCta;
    if (footerCta == null) return const SizedBox.shrink();
    return ListButton(
      text: Text(footerCta),
      dividerSide: DividerSide.top,
    );
  }

  Widget _buildBottomSection() => Container(alignment: Alignment.bottomCenter, child: bottomSection);
}
