import 'wallet_event.dart';

class EventSection {
  final DateTime dateTime;
  final List<WalletEvent> events;

  const EventSection(this.dateTime, this.events);
}
