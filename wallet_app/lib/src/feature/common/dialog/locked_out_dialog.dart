import 'dart:io';

import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';

class LockedOutDialog extends StatelessWidget {
  const LockedOutDialog({super.key});

  @override
  Widget build(BuildContext context) {
    final title = Platform.isIOS ? context.l10n.lockedOutDialogTitleiOSVariant : context.l10n.lockedOutDialogTitle;
    final description =
        Platform.isIOS ? context.l10n.lockedOutDialogDescriptioniOSVariant : context.l10n.lockedOutDialogDescription;
    return AlertDialog(
      title: Text.rich(title.toTextSpan(context)),
      content: Text.rich(description.toTextSpan(context)),
      actions: <Widget>[
        TextButton(
          child: Text.rich(context.l10n.generalOkCta.toUpperCase().toTextSpan(context)),
          onPressed: () => Navigator.pop(context),
        ),
      ],
    );
  }

  static Future<void> show(BuildContext context) {
    return showDialog<void>(
      context: context,
      builder: (BuildContext context) => const LockedOutDialog(),
    );
  }
}
