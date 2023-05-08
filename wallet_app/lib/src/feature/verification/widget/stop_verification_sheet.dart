import 'package:flutter/material.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../common/widget/button/link_button.dart';
import '../../common/widget/confirm_action_sheet.dart';

/// Builds upon the [ConfirmActionSheet], but supplies defaults for
/// when the user is requesting to stop the verification flow.
class StopVerificationSheet extends StatelessWidget {
  final String organizationName;
  final VoidCallback? onDataIncorrectPressed;
  final VoidCallback onConfirm;
  final VoidCallback onCancel;

  const StopVerificationSheet({
    required this.organizationName,
    this.onDataIncorrectPressed,
    required this.onCancel,
    required this.onConfirm,
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
      onCancel: onCancel,
      onConfirm: onConfirm,
      confirmIcon: Icons.not_interested,
      extraContent: onDataIncorrectPressed == null
          ? null
          : LinkButton(
              onPressed: onDataIncorrectPressed,
              customPadding: const EdgeInsets.all(16),
              child: Text(locale.stopVerificationSheetDataIncorrectCta),
            ),
    );
  }

  static Future<bool> show(
    BuildContext context, {
    required String organizationName,
    VoidCallback? onDataIncorrectPressed,
  }) async {
    final confirmed = await showModalBottomSheet<bool>(
      context: context,
      isScrollControlled: true,
      builder: (BuildContext context) {
        return StopVerificationSheet(
          organizationName: organizationName,
          onDataIncorrectPressed: onDataIncorrectPressed,
          onConfirm: () => Navigator.pop(context, true),
          onCancel: () => Navigator.pop(context, false),
        );
      },
    );
    return confirmed == true;
  }
}
