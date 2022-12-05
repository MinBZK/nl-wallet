import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../../domain/model/attribute/data_attribute.dart';
import '../../common/widget/check_data_offering_page.dart';
import '../../common/widget/confirm_buttons.dart';

class IssuanceCheckDataOfferingPage extends StatelessWidget {
  final VoidCallback onDecline;
  final VoidCallback onAccept;
  final List<DataAttribute> attributes;

  const IssuanceCheckDataOfferingPage({
    required this.onDecline,
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
      title: locale.issuanceCheckDataOfferingPageTitle,
      subtitle: locale.issuanceCheckDataOfferingPageSubtitle,
      footerCta: locale.issuanceCheckDataOfferingPageIncorrectCta,
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
