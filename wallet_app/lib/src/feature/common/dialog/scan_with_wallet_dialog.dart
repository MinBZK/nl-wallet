import 'package:flutter/material.dart';

import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../widget/text/body_text.dart';
import '../widget/text/title_text.dart';

class ScanWithWalletDialog extends StatelessWidget {
  const ScanWithWalletDialog({super.key});

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: TitleText(context.l10n.scanWithWalletDialogTitle),
      content: BodyText(context.l10n.scanWithWalletDialogBody),
      actions: <Widget>[
        TextButton(
          child: Text.rich(context.l10n.generalClose.toUpperCase().toTextSpan(context)),
          onPressed: () => Navigator.pop(context),
        ),
        TextButton(
          child: Text.rich(context.l10n.scanWithWalletDialogScanCta.toUpperCase().toTextSpan(context)),
          onPressed: () async {
            final navigator = Navigator.of(context);
            await navigator.pushNamedAndRemoveUntil(
              WalletRoutes.qrRoute,
              ModalRoute.withName(WalletRoutes.dashboardRoute),
            );
          },
        ),
      ],
    );
  }

  static Future<void> show(BuildContext context) {
    return showDialog<void>(
      context: context,
      builder: (BuildContext context) => const ScanWithWalletDialog(),
    );
  }
}
