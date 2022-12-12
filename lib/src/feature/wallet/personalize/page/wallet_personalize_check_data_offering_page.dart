import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../common/widget/check_data_offering_page.dart';

class WalletPersonalizeCheckDataOfferingPage extends StatelessWidget {
  final VoidCallback onAccept;
  final String name;
  final List<DataAttribute> attributes;

  const WalletPersonalizeCheckDataOfferingPage({
    required this.onAccept,
    required this.name,
    required this.attributes,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return CheckDataOfferingPage(
      bottomSection: _buildBottomSection(context),
      attributes: attributes,
      title: locale.walletPersonalizeCheckDataOfferingPageTitle(name),
      footerCta: locale.walletPersonalizeCheckDataOfferingPageIncorrectCta,
      showHeaderAttributesDivider: false,
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: ElevatedButton(
        onPressed: onAccept,
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.check, size: 16),
            const SizedBox(width: 8),
            Text(AppLocalizations.of(context).walletPersonalizeCheckDataOfferingPageContinueCta),
          ],
        ),
      ),
    );
  }
}
