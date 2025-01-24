import 'package:flutter/material.dart';

import '../../../domain/model/attribute/attribute.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../../common/page/check_data_offering_page.dart';
import '../../common/widget/button/confirm/confirm_buttons.dart';
import '../../common/widget/button/primary_button.dart';
import '../../common/widget/button/secondary_button.dart';

class IssuanceCheckDataOfferingPage extends StatelessWidget {
  final VoidCallback onDeclinePressed;
  final VoidCallback onAcceptPressed;
  final List<DataAttribute> attributes;

  const IssuanceCheckDataOfferingPage({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    required this.attributes,
    super.key,
  });

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
      primaryButton: PrimaryButton(
        key: const Key('acceptButton'),
        onPressed: onAcceptPressed,
        text: Text.rich(context.l10n.issuanceCheckDataOfferingPagePositiveCta.toTextSpan(context)),
        icon: const Icon(Icons.check),
      ),
      secondaryButton: SecondaryButton(
        key: const Key('rejectButton'),
        icon: null,
        onPressed: onDeclinePressed,
        text: Text.rich(context.l10n.issuanceCheckDataOfferingPageNegativeCta.toTextSpan(context)),
      ),
    );
  }
}
