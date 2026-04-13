import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';

/// A dialog that asks the user to confirm deletion of a card.
class DeleteCardDialog extends StatelessWidget {
  final String cardTitle;

  const DeleteCardDialog({required this.cardTitle, super.key});

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text.rich(
        context.l10n.deleteCardDialogTitle(cardTitle).toTextSpan(context),
        style: context.textTheme.headlineMedium,
      ),
      content: Text.rich(context.l10n.deleteCardDialogBody.toTextSpan(context)),
      actions: <Widget>[
        TextButton(
          child: Text.rich(context.l10n.deleteCardDialogCancelCta.toTextSpan(context)),
          onPressed: () => Navigator.pop(context, false),
        ),
        TextButton(
          style: TextButton.styleFrom(foregroundColor: context.colorScheme.error),
          child: Text.rich(context.l10n.deleteCardDialogConfirmCta.toTextSpan(context)),
          onPressed: () => Navigator.pop(context, true),
        ),
      ],
    );
  }

  /// Returns `true` if the user confirms deletion, `false` otherwise.
  static Future<bool> show(BuildContext context, {required String cardTitle}) async =>
      await showDialog<bool?>(
        context: context,
        builder: (BuildContext context) => DeleteCardDialog(cardTitle: cardTitle),
      ) ??
      false;
}
