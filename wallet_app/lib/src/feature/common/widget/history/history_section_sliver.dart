import 'package:flutter/material.dart';
import 'package:flutter_sticky_header/flutter_sticky_header.dart';

import '../../../../domain/model/event/event_section.dart';
import '../../../../domain/model/event/wallet_event.dart';
import 'history_section_header.dart';
import 'wallet_event_row.dart';

class HistorySectionSliver extends StatelessWidget {
  final EventSection section;
  final Function(WalletEvent event) onRowPressed;

  const HistorySectionSliver({
    required this.section,
    required this.onRowPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SliverStickyHeader(
      header: HistorySectionHeader(dateTime: section.dateTime),
      sliver: SliverList.builder(
        itemBuilder: (context, i) {
          final WalletEvent event = section.events[i];
          return Semantics(
            button: true,
            child: WalletEventRow(
              event: event,
              onPressed: () => onRowPressed(event),
            ),
          );
        },
        itemCount: section.events.length,
      ),
    );
  }
}
