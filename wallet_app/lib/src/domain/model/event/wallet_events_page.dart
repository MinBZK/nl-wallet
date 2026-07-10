import 'dart:collection';

import 'wallet_event.dart';

/// A page of [WalletEvent]s, merged with any previously loaded [pages].
class WalletEventsPage {
  final SplayTreeMap<int, List<WalletEvent>> pages;
  final bool hasNextPage;

  const WalletEventsPage({required this.pages, required this.hasNextPage});
}
