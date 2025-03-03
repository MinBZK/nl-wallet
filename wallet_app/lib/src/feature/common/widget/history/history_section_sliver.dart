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
      sliver: SliverList.separated(
        itemCount: section.events.length + 1, // +1 for the divider (separator) at the end of the list
        itemBuilder: (context, i) {
          return i < section.events.length
              ? Semantics(
                  button: true,
                  child: WalletEventRow(
                    event: section.events[i],
                    onPressed: () => onRowPressed(section.events[i]),
                  ),
                )
              : const SizedBox();
        },
        separatorBuilder: (context, index) => const Divider(),
      ),
    );
  }
}
