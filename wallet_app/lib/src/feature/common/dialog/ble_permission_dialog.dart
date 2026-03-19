import 'package:flutter/material.dart';
import 'package:permission_handler/permission_handler.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../widget/text/body_text.dart';
import '../widget/text/title_text.dart';

/// A dialog informing the user that Bluetooth permission is required
/// and offering the option to open the device settings.
class BlePermissionDialog extends StatelessWidget {
  const BlePermissionDialog({super.key});

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: TitleText(context.l10n.qrShowBluetoothPermissionTitle),
      content: BodyText(context.l10n.qrShowBluetoothPermissionDescription),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context, false),
          child: Text.rich(context.l10n.generalDialogCloseCta.toUpperCase().toTextSpan(context)),
        ),
        TextButton(
          onPressed: () => Navigator.pop(context, true),
          child: Text.rich(context.l10n.qrShowBluetoothPermissionSettingsCta.toUpperCase().toTextSpan(context)),
        ),
      ],
    );
  }

  /// Shows the [BlePermissionDialog].
  ///
  /// If the user chooses to open settings, the device settings are opened.
  static Future<void> show(BuildContext context) async {
    final openSettings =
        await showDialog<bool?>(
          context: context,
          builder: (BuildContext context) => const BlePermissionDialog(),
        ) ??
        false;
    if (openSettings) await openAppSettings();
  }
}
