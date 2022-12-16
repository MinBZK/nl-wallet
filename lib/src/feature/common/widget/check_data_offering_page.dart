import 'package:flutter/material.dart';

import '../../../domain/model/attribute/data_attribute.dart';
import 'attribute/data_attribute_row.dart';
import 'link_button.dart';
import 'placeholder_screen.dart';
import 'sliver_sized_box.dart';

/// Generic Page that displays the attributes so the user can check them.
/// Consumer needs to provide the [bottomSection] to handle any user actions.
class CheckDataOfferingPage extends StatelessWidget {
  final List<DataAttribute> attributes;
  final Widget bottomSection;
  final String title, footerCta;
  final String? subtitle;
  final bool showHeaderAttributesDivider;

  const CheckDataOfferingPage({
    required this.title,
    this.subtitle,
    this.showHeaderAttributesDivider = true,
    required this.footerCta,
    required this.bottomSection,
    required this.attributes,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      child: CustomScrollView(
        restorationId: 'check_data_offering_scrollview',
        slivers: <Widget>[
          const SliverSizedBox(height: 32),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          const SliverSizedBox(height: 24),
          if (showHeaderAttributesDivider) const SliverToBoxAdapter(child: Divider(height: 1)),
          const SliverSizedBox(height: 16),
          SliverList(delegate: _getDataAttributesDelegate()),
          const SliverSizedBox(height: 16),
          const SliverToBoxAdapter(child: Divider(height: 24)),
          SliverToBoxAdapter(child: _buildFooterSection(context)),
          const SliverToBoxAdapter(child: Divider(height: 24)),
          SliverFillRemaining(hasScrollBody: false, fillOverscroll: true, child: _buildBottomSection()),
        ],
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 0),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: Theme.of(context).textTheme.headline2,
          ),
          if (subtitle != null)
            Padding(
              padding: const EdgeInsets.only(top: 8),
              child: Text(
                subtitle!,
                style: Theme.of(context).textTheme.bodyText1,
                textAlign: TextAlign.center,
              ),
            )
        ],
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
    return Padding(
      padding: const EdgeInsets.only(left: 8.0),
      child: Align(
        alignment: AlignmentDirectional.centerStart,
        child: LinkButton(
          onPressed: () => PlaceholderScreen.show(context, footerCta),
          child: Text(footerCta),
        ),
      ),
    );
  }

  Widget _buildBottomSection() => Container(alignment: Alignment.bottomCenter, child: bottomSection);
}
