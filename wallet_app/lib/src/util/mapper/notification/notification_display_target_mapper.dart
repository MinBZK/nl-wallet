import 'package:wallet_core/core.dart' as core;

import '../../../domain/model/notification/app_notification.dart';
import '../mapper.dart';

class NotificationDisplayTargetMapper extends Mapper<core.DisplayTarget, NotificationDisplayTarget> {
  NotificationDisplayTargetMapper();

  @override
  NotificationDisplayTarget map(core.DisplayTarget input) {
    switch (input) {
      case core.DisplayTarget_Os():
        return NotificationDisplayTarget.os(notifyAt: DateTime.parse(input.notifyAt).toLocal());
      case core.DisplayTarget_Dashboard():
        return const NotificationDisplayTarget.dashboard();
    }
  }
}
