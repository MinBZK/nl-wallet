import 'package:flutter/material.dart';

import '../../../domain/model/event/wallet_event.dart';
import '../../extension/build_context_extension.dart';
import '../context_mapper.dart';

class WalletEventStatusColorMapper extends ContextMapper<WalletEvent, Color> {
  WalletEventStatusColorMapper();

  @override
  Color map(BuildContext context, WalletEvent input) {
    return switch (input) {
      DisclosureEvent() => mapDisclosureEvent(context, input),
      IssuanceEvent() => context.colorScheme.onSurface,
      SignEvent() => context.colorScheme.onSurface,
    };
  }

  Color mapDisclosureEvent(BuildContext context, DisclosureEvent input) {
    return switch (input.status) {
      EventStatus.success => context.colorScheme.onSurface,
      EventStatus.cancelled => context.colorScheme.error,
      EventStatus.error => context.colorScheme.error,
    };
  }
}
