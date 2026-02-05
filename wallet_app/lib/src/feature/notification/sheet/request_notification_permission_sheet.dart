import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../domain/usecase/notification/set_push_notifications_setting_usecase.dart';
import '../../../domain/usecase/permission/request_permission_usecase.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../common/sheet/confirm_action_sheet.dart';
import '../../common/widget/wallet_scrollbar.dart';

/// A bottom sheet that asks the user to grant notification permissions.
class RequestNotificationPermissionSheet extends StatelessWidget {
  const RequestNotificationPermissionSheet({
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return ConfirmActionSheet(
      title: context.l10n.requestNotificationPermissionSheetTitle,
      description: context.l10n.requestNotificationPermissionSheetDescription,
      confirmButton: ConfirmSheetButtonStyle(
        cta: context.l10n.requestNotificationPermissionSheetPositiveCta,
        icon: Icons.check_outlined,
      ),
      onConfirmPressed: () {
        final setPushNotificationsSettingUseCase = context.read<SetPushNotificationsSettingUseCase>();
        context.read<RequestPermissionUseCase>().invoke(.notification).then((result) {
          setPushNotificationsSettingUseCase.invoke(enabled: result.isGranted);
        });
        Navigator.pop(context);
      },
      cancelButton: ConfirmSheetButtonStyle(
        cta: context.l10n.requestNotificationPermissionSheetNegativeCta,
        icon: Icons.close_outlined,
      ),
      onCancelPressed: () => Navigator.pop(context),
    );
  }

  static Future<bool> show(BuildContext context) async {
    final confirmed = await showModalBottomSheet<bool>(
      context: context,
      isDismissible: false,
      isScrollControlled: true,
      builder: (BuildContext context) {
        return const WalletScrollbar(
          child: SingleChildScrollView(
            child: RequestNotificationPermissionSheet(),
          ),
        );
      },
    );
    return confirmed ?? false;
  }
}
