import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/card_front.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../../common/widget/status_icon.dart';
import '../../common/widget/wallet_card_front.dart';

class IssuanceCardAddedPage extends StatelessWidget {
  final VoidCallback onClose;
  final CardFront cardFront;

  const IssuanceCardAddedPage({
    required this.onClose,
    required this.cardFront,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return CustomScrollView(
      restorationId: 'proof_identity_scrollview',
      slivers: <Widget>[
        const SliverSizedBox(height: 48.0),
        SliverToBoxAdapter(child: _buildHeaderSection(context)),
        const SliverSizedBox(height: 32.0),
        SliverToBoxAdapter(child: _buildCardFront()),
        const SliverSizedBox(height: 16.0),
        SliverFillRemaining(hasScrollBody: false, fillOverscroll: true, child: _buildBottomSection(context)),
      ],
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16.0),
          child: StatusIcon(
            icon: Icons.check,
            color: Theme.of(context).primaryColor,
          ),
        ),
        const SizedBox(height: 32.0),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16.0),
          child: Text(
            locale.issuanceCardAddedTitle,
            style: Theme.of(context).textTheme.headline2,
            textAlign: TextAlign.center,
          ),
        ),
        const SizedBox(height: 8.0),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16.0),
          child: Text(
            locale.issuanceCardAddedSubtitle,
            style: Theme.of(context).textTheme.bodyText1,
            textAlign: TextAlign.center,
          ),
        ),
      ],
    );
  }

  Widget _buildCardFront() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16.0),
      child: WalletCardFront(
        cardFront: cardFront,
        onPressed: null,
      ),
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Align(
      alignment: Alignment.bottomCenter,
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: SizedBox(
          height: 48,
          child: ElevatedButton(
            onPressed: onClose,
            child: Text(AppLocalizations.of(context).issuanceCardAddedCloseCta),
          ),
        ),
      ),
    );
  }
}
