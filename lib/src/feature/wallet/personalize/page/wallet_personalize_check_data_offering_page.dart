import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../../domain/model/data_attribute.dart';
import '../../../common/widget/check_data_offering_page.dart';

class WalletPersonalizeCheckDataOfferingPage extends StatelessWidget {
  final VoidCallback onAccept;
  final List<DataAttribute> attributes;

  const WalletPersonalizeCheckDataOfferingPage({
    required this.onAccept,
    required this.attributes,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return CheckDataOfferingPage(
      bottomSection: _buildBottomSection(context),
      attributes: attributes,
      title: locale.walletPersonalizeCheckDataOfferingPageTitle,
      subtitle: locale.walletPersonalizeCheckDataOfferingPageSubtitle,
      footerCta: locale.walletPersonalizeCheckDataOfferingPageIncorrectCta,
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: ElevatedButton(
        onPressed: onAccept,
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.check, size: 16),
            const SizedBox(width: 8),
            Text(locale.walletPersonalizeCheckDataOfferingPageContinueCta),
          ],
        ),
      ),
    );
  }
}
