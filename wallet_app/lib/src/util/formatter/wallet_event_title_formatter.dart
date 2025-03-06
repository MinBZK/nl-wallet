import 'package:flutter/cupertino.dart';

import '../../domain/model/attribute/attribute.dart';
import '../../domain/model/event/wallet_event.dart';

class WalletEventTitleFormatter {
  /// Formats the title for [WalletEvent].
  static String format(BuildContext context, WalletEvent event) {
    switch (event) {
      case DisclosureEvent():
        return event.relyingParty.displayName.l10nValue(context);
      case IssuanceEvent():
        return event.card.title.l10nValue(context);
      case SignEvent():
        return event.relyingParty.displayName.l10nValue(context);
    }
  }
}
