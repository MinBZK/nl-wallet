import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/widget/attribute/attribute_row.dart';
import '../../common/widget/button/list_button.dart';
import '../../common/widget/sliver_divider.dart';
import '../../common/widget/sliver_sized_box.dart';

class DisclosureMissingAttributesPage extends StatelessWidget {
  final VoidCallback onDecline;
  final Organization organization;
  final List<Attribute> missingAttributes;

  const DisclosureMissingAttributesPage({
    required this.organization,
    required this.missingAttributes,
    required this.onDecline,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Scrollbar(
        child: CustomScrollView(
          restorationId: 'missing_data_attributes_scrollview',
          slivers: <Widget>[
            const SliverSizedBox(height: 32),
            SliverToBoxAdapter(child: _buildHeaderSection(context)),
            const SliverSizedBox(height: 32),
            const SliverDivider(height: 1),
            const SliverSizedBox(height: 24),
            SliverList(delegate: _getDataAttributesDelegate()),
            const SliverSizedBox(height: 24),
            SliverToBoxAdapter(child: _buildHowToProceedButton(context)),
            SliverFillRemaining(
              hasScrollBody: false,
              fillOverscroll: true,
              child: _buildCloseRequestButton(context),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            context.l10n.disclosureMissingAttributesPageTitle,
            style: context.textTheme.displayMedium,
            textAlign: TextAlign.start,
          ),
          const SizedBox(height: 8),
          Text(
            context.l10n.disclosureMissingAttributesPageDescription(organization.displayName.l10nValue(context)),
            style: context.textTheme.bodyLarge,
            textAlign: TextAlign.start,
          ),
        ],
      ),
    );
  }

  SliverChildBuilderDelegate _getDataAttributesDelegate() {
    return SliverChildBuilderDelegate(
      (context, index) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        child: AttributeRow(attribute: missingAttributes[index]),
      ),
      childCount: missingAttributes.length,
    );
  }

  Widget _buildHowToProceedButton(BuildContext context) {
    return ListButton(
      onPressed: () => PlaceholderScreen.show(context),
      text: Text(context.l10n.disclosureMissingAttributesPageHowToProceedCta),
    );
  }

  Widget _buildCloseRequestButton(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Align(
        alignment: Alignment.bottomCenter,
        child: ElevatedButton(
          onPressed: onDecline,
          child: Text(context.l10n.disclosureMissingAttributesPageCloseCta),
        ),
      ),
    );
  }
}
