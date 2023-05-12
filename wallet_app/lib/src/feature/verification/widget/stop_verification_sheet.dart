import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/button/link_button.dart';
import '../../common/widget/confirm_action_sheet.dart';

/// Builds upon the [ConfirmActionSheet], but supplies defaults for
/// when the user is requesting to stop the verification flow.
class StopVerificationSheet extends StatelessWidget {
  final String organizationName;
  final VoidCallback? onReportIssuePressed;
  final VoidCallback onCancelPressed;
  final VoidCallback onConfirmPressed;

  const StopVerificationSheet({
    required this.organizationName,
    this.onReportIssuePressed,
    required this.onCancelPressed,
    required this.onConfirmPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final locale = AppLocalizations.of(context);
    return ConfirmActionSheet(
      title: locale.stopVerificationSheetTitle,
      description: locale.stopVerificationSheetDescription(organizationName),
      cancelButtonText: locale.stopVerificationSheetNegativeCta,
      confirmButtonText: locale.stopVerificationSheetPositiveCta,
      confirmButtonColor: Theme.of(context).colorScheme.error,
      onCancelPressed: onCancelPressed,
      onConfirmPressed: onConfirmPressed,
      confirmIcon: Icons.not_interested,
      extraContent: onReportIssuePressed == null
          ? null
          : LinkButton(
              onPressed: onReportIssuePressed,
              customPadding: const EdgeInsets.all(16),
              child: Text(locale.stopVerificationSheetReportIssueCta),
            ),
    );
  }

  static Future<bool> show(
    BuildContext context, {
    required String organizationName,
    VoidCallback? onReportIssuePressed,
  }) async {
    final confirmed = await showModalBottomSheet<bool>(
      context: context,
      isScrollControlled: true,
      builder: (BuildContext context) {
        return StopVerificationSheet(
          organizationName: organizationName,
          onReportIssuePressed: onReportIssuePressed,
          onConfirmPressed: () => Navigator.pop(context, true),
          onCancelPressed: () => Navigator.pop(context, false),
        );
      },
    );
    return confirmed == true;
  }
}
