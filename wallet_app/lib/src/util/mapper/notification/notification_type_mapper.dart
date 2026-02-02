import 'package:wallet_core/core.dart' as core;

import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/notification/app_notification.dart';
import '../mapper.dart';

class NotificationTypeMapper extends Mapper<core.NotificationType, NotificationType> {
  final Mapper<core.AttestationPresentation, WalletCard> _cardMapper;

  NotificationTypeMapper(this._cardMapper);

  @override
  NotificationType map(core.NotificationType input) {
    switch (input) {
      case core.NotificationType_CardExpired():
        return CardExpired(card: _cardMapper.map(input.card));
      case core.NotificationType_CardExpiresSoon():
        return CardExpiresSoon(card: _cardMapper.map(input.card), expiresAt: DateTime.parse(input.expiresAt).toLocal());
      case core.NotificationType_Revoked():
        return CardRevoked(card: _cardMapper.map(input.card));
    }
  }
}
