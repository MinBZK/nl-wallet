import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../util/extension/string_extension.dart';

const _kRouteName = 'StopDigidLoginDialog';

/// A dialog that asks the user if they want to stop the DigID login process.
class StopDigidLoginDialog extends StatelessWidget {
  const StopDigidLoginDialog({super.key});

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text.rich(context.l10n.stopDigidLoginDialogTitle.toTextSpan(context)),
      content: Text.rich(context.l10n.stopDigidLoginDialogSubtitle.toTextSpan(context)),
      actions: <Widget>[
        TextButton(
          onPressed: () => Navigator.pop(context, false),
          child: Text.rich(context.l10n.stopDigidLoginDialogNegativeCta.toTextSpan(context)),
        ),
        TextButton(
          style: Theme.of(context)
              .textButtonTheme
              .style
              ?.copyWith(foregroundColor: WidgetStatePropertyAll(context.colorScheme.error)),
          onPressed: () => Navigator.pop(context, true),
          child: Text.rich(context.l10n.stopDigidLoginDialogPositiveCta.toTextSpan(context)),
        ),
      ],
    );
  }

  /// Shows the [StopDigidLoginDialog].
  ///
  /// Returns `true` if the user confirms to stop the login process, `false` otherwise.
  static Future<bool> show(BuildContext context) async =>
      await showDialog<bool?>(
        context: context,
        routeSettings: const RouteSettings(name: _kRouteName),
        builder: (BuildContext context) => const StopDigidLoginDialog(),
      ) ??
      false;

  static void closeOpenDialog(BuildContext context) =>
      Navigator.popUntil(context, (route) => route.settings.name != _kRouteName);
}
