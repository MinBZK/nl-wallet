import 'package:flutter/semantics.dart';

import '../../../l10n/generated/app_localizations.dart';
import '../../wallet_constants.dart';

class AnnouncementsHelper {
  AnnouncementsHelper._();

  static void announceEnteredDigits(AppLocalizations l10n, int enteredDigits) {
    SemanticsService.announce(
      l10n.pinEnteredDigitsAnnouncement(kPinDigits - enteredDigits),
      TextDirection.ltr,
      assertiveness: Assertiveness.assertive,
    );
  }
}
