import 'package:flutter/foundation.dart';

@immutable
class DeleteCardScreenArgument {
  static const _kAttestationIdKey = 'attestationId';
  static const _kCardTitleKey = 'cardTitle';

  final String attestationId;
  final String cardTitle;

  const DeleteCardScreenArgument({required this.attestationId, required this.cardTitle});

  Map<String, dynamic> toMap() {
    return {
      _kAttestationIdKey: attestationId,
      _kCardTitleKey: cardTitle,
    };
  }

  DeleteCardScreenArgument.fromMap(Map<String, dynamic> map)
    : attestationId = map[_kAttestationIdKey],
      cardTitle = map[_kCardTitleKey];

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is DeleteCardScreenArgument &&
          runtimeType == other.runtimeType &&
          attestationId == other.attestationId &&
          cardTitle == other.cardTitle;

  @override
  int get hashCode => Object.hash(
    runtimeType,
    attestationId,
    cardTitle,
  );
}
