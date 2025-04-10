import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../data/repository/wallet/wallet_repository.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';

class IdleWarningDialog extends StatelessWidget {
  const IdleWarningDialog({super.key});

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text.rich(context.l10n.idleWarningDialogTitle.toTextSpan(context)),
      content: Text.rich(context.l10n.idleWarningDialogDescription.toTextSpan(context)),
      actions: <Widget>[
        TextButton(
          child: Text.rich(context.l10n.idleWarningDialogLogoutCta.toUpperCase().toTextSpan(context)),
          onPressed: () {
            Navigator.pop(context);
            context.read<WalletRepository>().lockWallet();
          },
        ),
        TextButton(
          child: Text.rich(context.l10n.idleWarningDialogContinueCta.toUpperCase().toTextSpan(context)),
          onPressed: () => Navigator.pop(context),
        ),
      ],
    );
  }

  static Future<void> show(BuildContext context) {
    return showDialog<void>(
      context: context,
      builder: (BuildContext context) => const IdleWarningDialog(),
    );
  }
}
