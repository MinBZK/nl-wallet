import 'package:wallet_core/core.dart' as core;

import '../../../domain/model/notification/app_notification.dart';
import '../mapper.dart';

class AppNotificationMapper extends Mapper<core.AppNotification, AppNotification> {
  final Mapper<core.NotificationType, NotificationType> _typeMapper;
  final Mapper<core.DisplayTarget, NotificationDisplayTarget> _displayTargetMapper;

  AppNotificationMapper(this._typeMapper, this._displayTargetMapper);

  @override
  AppNotification map(core.AppNotification input) {
    return AppNotification(
      id: input.id,
      type: _typeMapper.map(input.typ),
      displayTargets: _displayTargetMapper.mapList(input.targets),
    );
  }
}
