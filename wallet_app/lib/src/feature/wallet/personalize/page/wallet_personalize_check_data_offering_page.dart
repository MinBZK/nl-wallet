import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../domain/model/attribute/ui_attribute.dart';
import '../../../common/widget/attribute/attribute_row.dart';
import '../../../common/widget/button/confirm_buttons.dart';
import '../../../common/widget/sliver_sized_box.dart';
import '../wallet_personalize_data_incorrect_screen.dart';

class WalletPersonalizeCheckDataOfferingPage extends StatelessWidget {
  final VoidCallback onAccept;
  final List<UiAttribute> attributes;

  const WalletPersonalizeCheckDataOfferingPage({
    required this.onAccept,
    required this.attributes,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      thumbVisibility: true,
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
    final locale = AppLocalizations.of(context);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 0),
      alignment: Alignment.centerLeft,
      child: Text(
        locale.walletPersonalizeCheckDataOfferingPageTitle,
        style: Theme.of(context).textTheme.displayMedium,
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
    final locale = AppLocalizations.of(context);
    return ConfirmButtons(
      onDecline: () => WalletPersonalizeDataIncorrectScreen.show(context),
      onAccept: onAccept,
      acceptText: locale.walletPersonalizeCheckDataOfferingPageAcceptCta,
      declineText: locale.walletPersonalizeCheckDataOfferingPageDeclineCta,
    );
  }
}
