import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';

/// A standardized Alert Dialog used across the application to display simple
/// informational messages with a single "OK" dismissal action.
///
/// Usage:
/// - Use [GenericDialog.show] for custom title/description dialogs.
/// - Use the static convenience methods (e.g., [showActiveDisclosureSession], [showFinishSetup])
///   for predefined application states that block navigation or actions.
class GenericDialog extends StatelessWidget {
  final String title;
  final String description;

  const GenericDialog({
    required this.title,
    required this.description,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text.rich(title.toTextSpan(context)),
      content: Text.rich(description.toTextSpan(context)),
      actions: <Widget>[
        TextButton(
          child: Text.rich(context.l10n.generalOkCta.toUpperCase().toTextSpan(context)),
          onPressed: () => Navigator.pop(context),
        ),
      ],
    );
  }

  /// Displays a generic dialog with a custom [title] and [description].
  static Future<void> show(BuildContext context, {required String title, required String description}) {
    return showDialog<void>(
      context: context,
      builder: (BuildContext context) => GenericDialog(title: title, description: description),
    );
  }

  /// Shows a dialog informing the user they must complete the current active
  /// disclosure session before proceeding.
  static Future<void> showActiveDisclosureSession(BuildContext context) => show(
    context,
    title: context.l10n.activeSessionDialogTitle,
    description: context.l10n.activeSessionDialogDescription,
  );

  /// Shows a dialog informing the user they must complete the current active
  /// issuance session before proceeding.
  static Future<void> showActiveIssuanceSession(BuildContext context) => show(
    context,
    title: context.l10n.activeIssuanceSessionDialogTitle,
    description: context.l10n.activeIssuanceSessionDialogDescription,
  );

  /// Shows a dialog informing the user they must finish the wallet setup process.
  static Future<void> showFinishSetup(BuildContext context) => show(
    context,
    title: context.l10n.finishSetupDialogTitle,
    description: context.l10n.finishSetupDialogDescription,
  );

  /// Shows a dialog informing the user they must complete the current data transfer. (source device)
  static Future<void> showFinishTransferSource(BuildContext context) => show(
    context,
    title: context.l10n.finishTransferSourceDialogTitle,
    description: context.l10n.finishTransferSourceDialogDescription,
  );

  /// Shows a dialog informing the user they must complete the current data transfer. (destination device)
  static Future<void> showFinishTransferDestination(BuildContext context) => show(
    context,
    title: context.l10n.finishTransferDestinationDialogTitle,
    description: context.l10n.finishTransferDestinationDialogDescription,
  );

  /// Shows a dialog informing the user they must complete the PIN recovery flow.
  static Future<void> showFinishPin(BuildContext context) => show(
    context,
    title: context.l10n.finishRecoverPinDialogTitle,
    description: context.l10n.finishRecoverPinDialogDescription,
  );
}
