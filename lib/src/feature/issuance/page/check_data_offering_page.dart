import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/data_attribute.dart';
import '../../common/widget/confirm_buttons.dart';
import '../../common/widget/data_attribute_row.dart';
import '../../common/widget/link_button.dart';
import '../../common/widget/placeholder_screen.dart';

class CheckDataOfferingPage extends StatelessWidget {
  final VoidCallback onDecline;
  final VoidCallback onAccept;
  final List<DataAttribute> attributes;

  const CheckDataOfferingPage({
    required this.onDecline,
    required this.onAccept,
    required this.attributes,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return CustomScrollView(
      restorationId: 'check_data_offering_scrollview',
      slivers: <Widget>[
        const SliverToBoxAdapter(child: SizedBox(height: 32)),
        SliverToBoxAdapter(child: _buildHeaderSection(context)),
        const SliverToBoxAdapter(child: SizedBox(height: 24)),
        const SliverToBoxAdapter(child: Divider(height: 1)),
        const SliverToBoxAdapter(child: SizedBox(height: 16)),
        SliverList(delegate: _getDataAttributesDelegate()),
        const SliverToBoxAdapter(child: Divider(height: 16)),
        SliverToBoxAdapter(child: _buildFooterSection(context)),
        const SliverToBoxAdapter(child: Divider(height: 16)),
        SliverFillRemaining(hasScrollBody: false, fillOverscroll: true, child: _buildBottomSection(context)),
      ],
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 0),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            locale.issuanceCheckDataOfferingPageTitle,
            style: Theme.of(context).textTheme.headline2,
          ),
          const SizedBox(height: 8),
          Text(
            locale.issuanceCheckDataOfferingPageSubtitle,
            style: Theme.of(context).textTheme.bodyText1,
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }

  SliverChildBuilderDelegate _getDataAttributesDelegate() {
    return SliverChildBuilderDelegate(
      (context, index) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: _buildDataAttributeItem(attributes[index]),
      ),
      childCount: attributes.length,
    );
  }

  Widget _buildDataAttributeItem(DataAttribute attribute) {
    return DataAttributeRow(attribute: attribute);
  }

  Widget _buildFooterSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(left: 8.0),
      child: Align(
        alignment: AlignmentDirectional.centerStart,
        child: LinkButton(
          onPressed: () => PlaceholderScreen.show(
            context,
            AppLocalizations.of(context).issuanceCheckDataOfferingPageIncorrectCta,
          ),
          child: Text(AppLocalizations.of(context).issuanceCheckDataOfferingPageIncorrectCta),
        ),
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return ConfirmButtons(
      onAccept: onAccept,
      acceptText: locale.issuanceCheckDataOfferingPagePositiveCta,
      onDecline: onDecline,
      declineText: locale.issuanceCheckDataOfferingPageNegativeCta,
      acceptIcon: Icons.check,
    );
  }
}
