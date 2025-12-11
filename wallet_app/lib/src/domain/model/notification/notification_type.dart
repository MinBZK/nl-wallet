import 'package:freezed_annotation/freezed_annotation.dart';

import '../card/wallet_card.dart';

part 'notification_type.freezed.dart';

@freezed
sealed class NotificationType with _$NotificationType {
  const factory NotificationType.cardExpiresSoon({
    required WalletCard card,
    required DateTime expiresAt,
  }) = CardExpiresSoon;

  const factory NotificationType.cardExpired({
    required WalletCard card,
  }) = CardExpired;
}
