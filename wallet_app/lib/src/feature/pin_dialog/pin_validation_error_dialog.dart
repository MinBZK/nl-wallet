import 'dart:io';

import 'package:flutter/material.dart';

import '../../domain/model/pin/pin_validation_error.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/string_extension.dart';

class PinValidationErrorDialog extends StatelessWidget {
  final PinValidationError reason;

  const PinValidationErrorDialog({required this.reason, super.key});

  @override
  Widget build(BuildContext context) {
    final title = switch (reason) {
      PinValidationError.tooFewUniqueDigits => context.l10n.pinValidationErrorDialogTitle,
      PinValidationError.sequentialDigits => context.l10n.pinValidationErrorDialogTitle,
      PinValidationError.other => context.l10n.pinValidationErrorDialogTitle,
    };
    final body = switch (reason) {
      PinValidationError.tooFewUniqueDigits => context.l10n.pinValidationErrorDialogTooFewUniqueDigitsError,
      PinValidationError.sequentialDigits => context.l10n.pinValidationErrorDialogAscendingOrDescendingDigitsError,
      PinValidationError.other => context.l10n.pinValidationErrorDialogDefaultError,
    };
    return AlertDialog(
      scrollable: true,
      semanticLabel: Platform.isAndroid ? title : null,
      title: Text(title),
      content: Text(body),
      actions: <Widget>[
        TextButton(
          child: Text.rich(context.l10n.generalOkCta.toTextSpan(context)),
          onPressed: () => Navigator.pop(context),
        ),
      ],
    );
  }

  static Future<void> show(BuildContext context, PinValidationError reason) {
    return showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (BuildContext context) => PinValidationErrorDialog(reason: reason),
    );
  }
}
