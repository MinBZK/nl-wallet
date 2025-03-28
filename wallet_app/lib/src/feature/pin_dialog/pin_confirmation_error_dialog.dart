import 'dart:io';

import 'package:flutter/material.dart';

import '../../util/extension/build_context_extension.dart';

class PinConfirmationErrorDialog extends StatelessWidget {
  final bool retryAllowed;

  const PinConfirmationErrorDialog({required this.retryAllowed, super.key});

  @override
  Widget build(BuildContext context) {
    final title =
        retryAllowed ? context.l10n.pinConfirmationErrorDialogTitle : context.l10n.pinConfirmationErrorDialogFatalTitle;
    final content = retryAllowed
        ? context.l10n.pinConfirmationErrorDialogDescription
        : context.l10n.pinConfirmationErrorDialogFatalDescription;
    final cta = retryAllowed ? context.l10n.generalOkCta : context.l10n.pinConfirmationErrorDialogFatalCta;

    return AlertDialog(
      scrollable: true,
      semanticLabel: Platform.isAndroid ? title : null,
      title: Text(title),
      content: Text(content),
      actions: <Widget>[
        TextButton(
          child: Text(cta),
          onPressed: () => Navigator.pop(context),
        ),
      ],
    );
  }

  static Future<void> show(BuildContext context, {required bool retryAllowed}) {
    return showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (BuildContext context) => PinConfirmationErrorDialog(retryAllowed: retryAllowed),
    );
  }
}
