import 'dart:ui';

import 'package:flutter/cupertino.dart';
import 'package:flutter/semantics.dart';

import '../../../l10n/generated/app_localizations.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_constants.dart';

class AnnouncementService {
  final BuildContext _context;

  FlutterView get _view => View.of(_context);

  bool get announcementsEnabled => _context.isScreenReaderEnabled;

  const AnnouncementService(this._context);

  Future<void> announce(String text, {Assertiveness assertiveness = Assertiveness.polite}) async {
    if (!announcementsEnabled) return;
    return SemanticsService.sendAnnouncement(_view, text, .ltr, assertiveness: assertiveness);
  }

  Future<void> announceEnteredDigits(AppLocalizations l10n, int enteredDigits) async {
    if (!announcementsEnabled) return;
    final announcement = l10n.pinEnteredDigitsAnnouncement(kPinDigits - enteredDigits);
    return SemanticsService.sendAnnouncement(_view, announcement, .ltr, assertiveness: .assertive);
  }
}
