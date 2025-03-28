import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/wallet/reset_wallet_usecase.dart';
import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';

class ResetWalletDialog extends StatelessWidget {
  const ResetWalletDialog({super.key});

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text.rich(
        context.l10n.resetWalletDialogTitle.toTextSpan(context),
        style: context.textTheme.headlineMedium,
      ),
      content: Text.rich(context.l10n.resetWalletDialogBody.toTextSpan(context)),
      actions: <Widget>[
        TextButton(
          child: Text.rich(context.l10n.resetWalletDialogCancelCta.toTextSpan(context)),
          onPressed: () => Navigator.pop(context),
        ),
        TextButton(
          style: TextButton.styleFrom(foregroundColor: context.colorScheme.error),
          child: Text.rich(context.l10n.resetWalletDialogConfirmCta.toTextSpan(context)),
          onPressed: () async {
            final navigator = Navigator.of(context);
            await context.read<ResetWalletUseCase>().invoke();
            await navigator.pushNamedAndRemoveUntil(
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
