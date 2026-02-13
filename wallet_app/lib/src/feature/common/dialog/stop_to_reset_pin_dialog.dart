import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';

const _kRouteName = 'StopToResetPinDialog';

/// A dialog that prompts the user to confirm whether they want to interrupt
/// their current activity (session) to proceed with resetting their PIN.
///
/// This dialog is context-aware and adjusts its text based on the type of
/// [StopCopyVariant] provided.
class StopToResetPinDialog extends StatelessWidget {
  /// The type of activity currently in progress (e.g., disclosure, issuance).
  final StopCopyVariant variant;

  /// The name of the organization involved in the session, if applicable.
  /// Used to personalize the dialog message for 'disclosure' and 'login' variants.
  final String? organizationName;

  const StopToResetPinDialog({
    required this.variant,
    this.organizationName,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text.rich(_resolveTitle(context).toTextSpan(context)),
      content: Text.rich(_resolveDescription(context).toTextSpan(context)),
      actions: <Widget>[
        TextButton(
          onPressed: () => Navigator.pop(context, false),
          child: Text.rich(context.l10n.stopToResetPinDialogCancelCta.toUpperCase().toTextSpan(context)),
        ),
        TextButton(
          style: Theme.of(
            context,
          ).textButtonTheme.style?.copyWith(foregroundColor: WidgetStatePropertyAll(context.colorScheme.error)),
          onPressed: () => Navigator.pop(context, true),
          child: Text.rich(context.l10n.stopToResetPinDialogConfirmCta.toUpperCase().toTextSpan(context)),
        ),
      ],
    );
  }

  String _resolveTitle(BuildContext context) => switch (variant) {
    StopCopyVariant.disclosure => context.l10n.stopToResetPinDialogTitleDisclosureVariant,
    StopCopyVariant.issuance => context.l10n.stopToResetPinDialogTitleIssuanceVariant,
    StopCopyVariant.login => context.l10n.stopToResetPinDialogTitleLoginVariant,
    StopCopyVariant.transfer => context.l10n.stopToResetPinDialogTitleTransferVariant,
  };

  String _resolveDescription(BuildContext context) => switch (variant) {
    StopCopyVariant.disclosure => context.l10n.stopToResetPinDialogDescriptionDisclosureVariant(
      organizationName ?? context.l10n.organizationFallbackName,
    ),
    StopCopyVariant.issuance => context.l10n.stopToResetPinDialogDescriptionIssuanceVariant,
    StopCopyVariant.login => context.l10n.stopToResetPinDialogDescriptionLoginVariant(
      organizationName ?? context.l10n.organizationFallbackName,
    ),
    StopCopyVariant.transfer => context.l10n.stopToResetPinDialogDescriptionTransferVariant,
  };

  /// Displays the [StopToResetPinDialog].
  ///
  /// [variant] determines the specific messaging shown to the user.
  /// [organizationName] is an optional string to identify the third party involved.
  ///
  /// Returns `true` if the user confirms they want to stop the process and reset their PIN,
  /// and `false` if they cancel or dismiss the dialog.
  static Future<bool> show(BuildContext context, StopCopyVariant variant, {String? organizationName}) async =>
      await showDialog<bool?>(
        context: context,
        routeSettings: const RouteSettings(name: _kRouteName),
        builder: (BuildContext context) => StopToResetPinDialog(
          variant: variant,
          organizationName: organizationName,
        ),
      ) ??
      false;

  static void closeOpenDialog(BuildContext context) =>
      Navigator.popUntil(context, (route) => route.settings.name != _kRouteName);
}

/// Represents the different types of user flows that can be interrupted by a PIN reset request.
enum StopCopyVariant {
  /// The user is currently sharing attributes with a third party.
  disclosure,

  /// The user is in the process of receiving a new credential.
  issuance,

  /// The user is performing an login flow.
  login,

  /// The user is transferring wallet data to a new device.
  transfer,
}
