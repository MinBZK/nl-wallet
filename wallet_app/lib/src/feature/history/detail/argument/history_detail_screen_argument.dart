import 'package:flutter/foundation.dart';

import '../../../../domain/model/event/wallet_event.dart';

@immutable
class HistoryDetailScreenArgument {
  static const _kWalletEventKey = 'walletEvent';
  static const _kCardDocType = 'docType';

  final WalletEvent walletEvent;
  final String? docType;

  const HistoryDetailScreenArgument({required this.walletEvent, this.docType});

  Map<String, dynamic> toMap() {
    return {
      _kWalletEventKey: walletEvent,
      _kCardDocType: docType,
    };
  }

  HistoryDetailScreenArgument.fromMap(Map<String, dynamic> map)
      : walletEvent = map[_kWalletEventKey],
        docType = map[_kCardDocType];

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is HistoryDetailScreenArgument &&
          runtimeType == other.runtimeType &&
          walletEvent == other.walletEvent &&
          docType == other.docType;

  @override
  int get hashCode => Object.hash(
        runtimeType,
        walletEvent,
        docType,
      );
}
