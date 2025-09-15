import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/sheet/confirm_action_sheet.dart';
import '../../common/widget/wallet_scrollbar.dart';

/// Builds upon the [ConfirmActionSheet], but supplies defaults for
/// when the user is requesting to stop the wallet transfer on the source device.
class WalletTransferSourceStopSheet extends StatelessWidget {
  /// Callback invoked when the confirm action (stop) is pressed.
  final VoidCallback onConfirmPressed;

  /// Callback invoked when the cancel action is pressed.
  final VoidCallback onCancelPressed;

  const WalletTransferSourceStopSheet({
    required this.onCancelPressed,
    required this.onConfirmPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return ConfirmActionSheet(
      title: context.l10n.walletTransferSourceStopSheetTitle,
      description: context.l10n.walletTransferSourceStopSheetDescription,
      confirmButton: ConfirmSheetButtonStyle(
        cta: context.l10n.walletTransferSourceStopSheetConfirmCta,
        color: context.colorScheme.error,
        icon: Icons.not_interested,
      ),
      cancelButton: ConfirmSheetButtonStyle(
        cta: context.l10n.walletTransferSourceStopSheetCancelCta,
        icon: Icons.arrow_back,
      ),
      onCancelPressed: onCancelPressed,
      onConfirmPressed: onConfirmPressed,
    );
  }

  /// Shows a modal bottom sheet to confirm stopping the wallet transfer.
  ///
  /// Returns `true` if the user really wants to stop, and `false` otherwise.
  static Future<bool> show(BuildContext context) async {
    final confirmed = await showModalBottomSheet<bool>(
      context: context,
      isDismissible: !context.isScreenReaderEnabled, // Avoid announcing the scrim
      isScrollControlled: true,
      builder: (BuildContext context) {
        return WalletScrollbar(
          child: SingleChildScrollView(
            child: WalletTransferSourceStopSheet(
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
