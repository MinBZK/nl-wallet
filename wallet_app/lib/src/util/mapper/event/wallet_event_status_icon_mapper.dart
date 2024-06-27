import 'package:flutter/material.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../mapper.dart';

class WalletEventStatusIconMapper extends Mapper<WalletEvent, IconData?> {
  WalletEventStatusIconMapper();

  @override
  IconData? map(WalletEvent input) {
    return switch (input.status) {
      EventStatus.success => null,
      EventStatus.cancelled => Icons.block_flipped,
      EventStatus.error => Icons.error_outline_outlined,
    };
  }
}
