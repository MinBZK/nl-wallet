import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/wallet/reset_wallet_usecase.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';

class ResetWalletDialog extends StatelessWidget {
  const ResetWalletDialog({super.key});

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text(context.l10n.resetWalletDialogTitle),
      content: Text(context.l10n.resetWalletDialogBody),
      actions: <Widget>[
        TextButton(
          child: Text(context.l10n.resetWalletDialogCancelCta),
          onPressed: () => Navigator.pop(context),
        ),
        TextButton(
          style: TextButton.styleFrom(foregroundColor: context.colorScheme.error),
          child: Text(context.l10n.resetWalletDialogConfirmCta),
          onPressed: () async {
            final navigator = Navigator.of(context);
            await context.read<ResetWalletUseCase>().invoke();
            navigator.pushNamedAndRemoveUntil(
              WalletRoutes.splashRoute,
              ModalRoute.withName(WalletRoutes.splashRoute),
            );
          },
        ),
      ],
    );
  }

  static Future<void> show(BuildContext context) {
    return showDialog<void>(
      context: context,
      builder: (BuildContext context) => const ResetWalletDialog(),
    );
  }
}
