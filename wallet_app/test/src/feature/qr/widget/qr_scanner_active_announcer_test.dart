import 'package:flutter/semantics.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:semantic_announcement_tester/semantic_announcement_tester.dart';
import 'package:wallet/src/feature/qr/widget/qr_scanner_active_announcer.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/test_utils.dart';

main() {
  group('widgets', () {
    testWidgets('No QR code announcement is made after 10 seconds', (tester) async {
      final mock = MockSemanticAnnouncements(tester);
      final l10n = await TestUtils.englishLocalizations;
      final expectedAnnouncement = AnnounceSemanticsEvent(
        l10n.qrScreenScanScannerActiveWCAGAnnouncement,
        TextDirection.ltr,
      );
      await tester.pumpWidgetWithAppWrapper(const QrScannerActiveAnnouncer());
      await tester.pump(const Duration(seconds: 10));
      await tester.pumpAndSettle();

      expect(mock.announcements, hasOneAnnouncement(expectedAnnouncement));
    });

    testWidgets('No QR code announcement is made repeatedly every 10 seconds', (tester) async {
      final mock = MockSemanticAnnouncements(tester);
      final l10n = await TestUtils.englishLocalizations;
      final expectedAnnouncement = AnnounceSemanticsEvent(
        l10n.qrScreenScanScannerActiveWCAGAnnouncement,
        TextDirection.ltr,
      );

      final expectedAnnouncements = List.generate(9, (i) => expectedAnnouncement);

      await tester.pumpWidgetWithAppWrapper(const QrScannerActiveAnnouncer());
      await tester.pump(Duration(seconds: expectedAnnouncements.length * 10 /* 1 announcement every 10 seconds */));
      await tester.pumpAndSettle();

      expect(mock.announcements, hasNAnnouncements(expectedAnnouncements));
    });
  });
}
