import 'package:flutter/foundation.dart';

import '../../../../domain/model/event/wallet_event.dart';

@immutable
class HistoryDetailScreenArgument {
  static const _kWalletEventKey = 'walletEvent';

  final WalletEvent walletEvent;

  const HistoryDetailScreenArgument({required this.walletEvent});

  Map<String, dynamic> toMap() {
    return {_kWalletEventKey: walletEvent};
  }

  HistoryDetailScreenArgument.fromMap(Map<String, dynamic> map) : walletEvent = map[_kWalletEventKey];

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is HistoryDetailScreenArgument && runtimeType == other.runtimeType && walletEvent == other.walletEvent;

  @override
  int get hashCode => Object.hash(
    runtimeType,
    walletEvent,
  );
}
