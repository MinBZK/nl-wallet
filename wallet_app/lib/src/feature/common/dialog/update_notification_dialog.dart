import 'dart:io';

import 'package:flutter/material.dart';
import 'package:store_redirect/store_redirect.dart';

import '../../../navigation/wallet_routes.dart';
import '../../../util/extension/build_context_extension.dart';
import '../widget/text/body_text.dart';
import '../widget/text/title_text.dart';

class UpdateNotificationDialog extends StatelessWidget {
  final Duration? timeUntilBlocked;

  const UpdateNotificationDialog({this.timeUntilBlocked, super.key});

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: TitleText(
        context.l10n.updateNotificationDialogTitle,
        style: context.textTheme.displayMedium,
      ),
      content: BodyText(
        _resolveDescription(context),
      ),
      actions: <Widget>[
        TextButton(
          child: Text(context.l10n.generalClose.toUpperCase()),
          onPressed: () => Navigator.pop(context),
        ),
        TextButton(
          child: Text(context.l10n.generalNeedHelpCta.toUpperCase()),
          onPressed: () async {
            final navigator = Navigator.of(context);
            await navigator.pushNamed(WalletRoutes.updateInfoRoute);
          },
        ),
        TextButton(
          child: Text(
            (Platform.isIOS ? context.l10n.generalToAppStoreCta : context.l10n.generalToPlayStoreCta).toUpperCase(),
          ),
          onPressed: () => StoreRedirect.redirect,
        ),
      ],
    );
  }

  String _resolveDescription(BuildContext context) {
    final timeUntilBlocked = this.timeUntilBlocked;
    final appStore = Platform.isIOS ? context.l10n.generalAppStoreiOS : context.l10n.generalAppStoreAndroid;
    if (timeUntilBlocked == null) {
      return context.l10n.updateNotificationDialogDescription(appStore);
    } else {
      final days = timeUntilBlocked.inDays;
      final hours = timeUntilBlocked.inHours;
      if (days > 0) {
        return context.l10n.updateNotificationDialogCountdownDescription(appStore, context.l10n.generalDays(days));
      } else if (hours > 0) {
        return context.l10n.updateNotificationDialogCountdownDescription(appStore, context.l10n.generalHours(hours));
      } else {
        return context.l10n.updateNotificationDialogCountdownDescription(appStore, context.l10n.generalHours(1));
      }
    }
  }

  static Future<void> show(BuildContext context, {Duration? timeUntilBlocked}) {
    return showDialog<void>(
      context: context,
      builder: (BuildContext context) => UpdateNotificationDialog(timeUntilBlocked: timeUntilBlocked),
    );
  }
}
