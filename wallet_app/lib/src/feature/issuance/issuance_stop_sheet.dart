import 'package:flutter/material.dart';

import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';
import '../common/sheet/confirm_action_sheet.dart';
import '../common/widget/button/list_button.dart';
import '../common/widget/wallet_scrollbar.dart';

/// Builds upon the [ConfirmActionSheet], but supplies defaults for
/// when the user is requesting to stop the issuance flow.
class IssuanceStopSheet extends StatelessWidget {
  final String? organizationName;
  final VoidCallback? onReportIssuePressed;
  final VoidCallback onCancelPressed;
  final VoidCallback onConfirmPressed;

  const IssuanceStopSheet({
    required this.organizationName,
    this.onReportIssuePressed,
    required this.onCancelPressed,
    required this.onConfirmPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return ConfirmActionSheet(
      title: context.l10n.issuanceStopSheetTitle,
      description: context.l10n.issuanceStopSheetDescription(organizationName ?? context.l10n.organizationFallbackName),
      confirmButtonText: context.l10n.issuanceStopSheetPositiveCta,
      confirmButtonColor: context.colorScheme.error,
      confirmIcon: Icons.block_flipped,
      cancelButtonText: context.l10n.issuanceStopSheetNegativeCta,
      cancelIcon: Icons.arrow_back,
      extraContent: ListButton(
        dividerSide: DividerSide.none,
        onPressed: onReportIssuePressed,
        text: Text.rich(context.l10n.issuanceStopSheetReportIssueCta.toTextSpan(context)),
      ),
      onCancelPressed: onCancelPressed,
      onConfirmPressed: onConfirmPressed,
    );
  }

  /// Shows a modal bottom sheet to confirm stopping the issuance flow.
  ///
  /// Returns a [Future] that resolves to `true` if the user confirms stopping
  /// the issuance, and `false` if the user cancels or dismisses the sheet.
  ///
  /// The [context] is the `BuildContext` from which to show the sheet.
  /// The [organizationName] is the optional name of the organization, which
  /// will be displayed in the sheet's description.
  /// The [onReportIssuePressed] is an optional callback that is triggered when
  /// the "Report issue" button is pressed. This button is only shown when the
  /// callback is provided.
  static Future<bool> show(
    BuildContext context, {
    String? organizationName,
    VoidCallback? onReportIssuePressed,
  }) async {
    final confirmed = await showModalBottomSheet<bool>(
      context: context,
      isDismissible: !context.isScreenReaderEnabled, // Avoid announcing the scrim
      isScrollControlled: true,
      builder: (BuildContext context) {
        return WalletScrollbar(
          child: SingleChildScrollView(
            child: IssuanceStopSheet(
              organizationName: organizationName,
              onReportIssuePressed: onReportIssuePressed,
              onConfirmPressed: () => Navigator.pop(context, true),
              onCancelPressed: () => Navigator.pop(context, false),
            ),
          ),
        );
      },
    );
    return confirmed ?? false;
  }
}
