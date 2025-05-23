import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/organization.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../widget/attribute/attribute_row.dart';
import '../widget/button/list_button.dart';
import '../widget/spacer/sliver_divider.dart';
import '../widget/spacer/sliver_sized_box.dart';
import '../widget/text/body_text.dart';
import '../widget/text/title_text.dart';
import '../widget/wallet_scrollbar.dart';

class MissingAttributesPage extends StatelessWidget {
  final VoidCallback onDecline;
  final VoidCallback onReportIssuePressed;
  final Organization organization;
  final List<Attribute> missingAttributes;

  const MissingAttributesPage({
    required this.organization,
    required this.missingAttributes,
    required this.onDecline,
    required this.onReportIssuePressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: WalletScrollbar(
        child: CustomScrollView(
          restorationId: 'missing_data_attributes_scrollview',
          slivers: <Widget>[
            const SliverSizedBox(height: 32),
            SliverToBoxAdapter(child: _buildHeaderSection(context)),
            const SliverSizedBox(height: 32),
            const SliverDivider(),
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
          TitleText(context.l10n.missingAttributesPageTitle),
          const SizedBox(height: 8),
          BodyText(
            context.l10n.missingAttributesPageDescription(
              organization.displayName.l10nValue(context),
            ),
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
      onPressed: onReportIssuePressed,
      text: Text.rich(context.l10n.missingAttributesPageReportIssueCta.toTextSpan(context)),
    );
  }

  Widget _buildCloseRequestButton(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Align(
        alignment: Alignment.bottomCenter,
        child: ElevatedButton(
          onPressed: onDecline,
          child: Text.rich(context.l10n.missingAttributesPageCloseCta.toTextSpan(context)),
        ),
      ),
    );
  }
}
