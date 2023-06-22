import 'package:flutter/material.dart';

import '../../../domain/model/attribute/data_attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/button/confirm_buttons.dart';
import '../../common/widget/check_data_offering_page.dart';

class IssuanceCheckDataOfferingPage extends StatelessWidget {
  final VoidCallback onDeclinePressed;
  final VoidCallback onAcceptPressed;
  final List<DataAttribute> attributes;

  const IssuanceCheckDataOfferingPage({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    required this.attributes,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return CheckDataOfferingPage(
      bottomSection: _buildBottomSection(context),
      attributes: attributes,
      title: context.l10n.issuanceCheckDataOfferingPageTitle,
      subtitle: context.l10n.issuanceCheckDataOfferingPageSubtitle,
    );
  }

  Widget _buildBottomSection(BuildContext context) {
    return ConfirmButtons(
      onAcceptPressed: onAcceptPressed,
      acceptText: context.l10n.issuanceCheckDataOfferingPagePositiveCta,
      onDeclinePressed: onDeclinePressed,
      declineText: context.l10n.issuanceCheckDataOfferingPageNegativeCta,
      acceptIcon: Icons.check,
    );
  }
}
