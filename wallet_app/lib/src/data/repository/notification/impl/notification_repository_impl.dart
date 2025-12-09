import 'package:wallet_core/core.dart' as core;

import '../../../../domain/model/notification/app_notification.dart';
import '../../../../util/mapper/mapper.dart';
import '../../../../wallet_core/typed/typed_wallet_core.dart';
import '../notification_repository.dart';

class NotificationRepositoryImpl implements NotificationRepository {
  final TypedWalletCore _core;
  final Mapper<core.AppNotification, AppNotification> _notificationMapper;

  NotificationRepositoryImpl(this._core, this._notificationMapper);

  @override
  Stream<List<AppNotification>> observeNotifications() => _core.observeNotifications().map(_notificationMapper.mapList);
}
