import 'package:collection/collection.dart';

import '../../domain/model/attribute/attribute.dart';
import '../../domain/model/event/event_section.dart';
import '../../domain/model/event/wallet_event.dart';
import '../../domain/model/organization.dart';
import 'date_time_extension.dart';

extension WalletEventExtensions on WalletEvent {
  Map<String, List<DataAttribute>> get attributesByDocType => groupBy(attributes, (attr) => attr.sourceCardDocType);

  Organization get relyingPartyOrIssuer => switch (this) {
        DisclosureEvent() => (this as DisclosureEvent).relyingParty,
        IssuanceEvent() => (this as IssuanceEvent).card.issuer,
        SignEvent() => (this as SignEvent).relyingParty,
      };

  bool get wasSuccess => status == EventStatus.success;

  bool get wasCancelled => status == EventStatus.cancelled;

  bool get wasFailure => status == EventStatus.error;
}

extension WalletEventListExtensions on List<WalletEvent> {
  List<EventSection> get sectionedByMonth {
    return groupListsBy((element) => element.dateTime.yearMonth)
        .entries
        .map((e) => EventSection(e.key, e.value))
        .toList();
  }
}
