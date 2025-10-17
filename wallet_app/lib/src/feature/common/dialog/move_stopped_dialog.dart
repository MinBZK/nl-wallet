import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';
import '../widget/text/body_text.dart';
import '../widget/text/title_text.dart';

class MoveStoppedDialog extends StatelessWidget {
  const MoveStoppedDialog({super.key});

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: TitleText(context.l10n.moveStoppedDialogTitle),
      content: BodyText(context.l10n.moveStoppedDialogBody),
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
      builder: (BuildContext context) => const MoveStoppedDialog(),
    );
  }
}
