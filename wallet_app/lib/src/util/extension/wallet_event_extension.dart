import 'package:collection/collection.dart';

import '../../domain/model/event/event_section.dart';
import '../../domain/model/event/wallet_event.dart';
import 'date_time_extension.dart';

extension WalletEventExtensions on WalletEvent {
  bool get wasSuccess => status == EventStatus.success;

  bool get wasCancelled => status == EventStatus.cancelled;

  bool get wasFailure => status == EventStatus.error;
}

extension WalletEventListExtensions on List<WalletEvent> {
  List<EventSection> get sectionedByMonth {
    return groupListsBy(
      (element) => element.dateTime.yearMonth,
    ).entries.map((e) => EventSection(e.key, e.value)).toList();
  }
}
