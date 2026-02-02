import 'package:freezed_annotation/freezed_annotation.dart';

part 'card_status.freezed.dart';
part 'card_status.g.dart';

@freezed
sealed class CardStatus with _$CardStatus {
  const factory CardStatus.validSoon({
    /// Time from which the card is valid
    required DateTime validFrom,
  }) = CardStatusValidSoon;

  const factory CardStatus.valid({
    /// Time until the card is valid (expiry date)
    required DateTime? validUntil,
  }) = CardStatusValid;

  const factory CardStatus.expiresSoon({
    /// Time until the card is valid (expiry date)
    required DateTime validUntil,
  }) = CardStatusExpiresSoon;

  const factory CardStatus.expired({
    /// Time until the card is valid (expiry date)
    required DateTime validUntil,
  }) = CardStatusExpired;

  const factory CardStatus.revoked() = CardStatusRevoked;

  const factory CardStatus.corrupted() = CardStatusCorrupted;

  const factory CardStatus.undetermined() = CardStatusUndetermined;

  factory CardStatus.fromJson(Map<String, dynamic> json) => _$CardStatusFromJson(json);
}
