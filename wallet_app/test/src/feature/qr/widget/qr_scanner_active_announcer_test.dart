import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/announcement_service.dart';
import 'package:wallet/src/feature/qr/widget/qr_scanner_active_announcer.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../test_util/test_utils.dart';

void main() {
  group('widgets', () {
    testWidgets('ltc7 ltc16 ltc19 No QR code announcement is made after 10 seconds', (tester) async {
      final mock = MockAnnouncementService();
      final l10n = await TestUtils.englishLocalizations;
      final expectedAnnouncement = l10n.qrScreenScanScannerActiveWCAGAnnouncement;

      await tester.pumpWidgetWithAppWrapper(
        const QrScannerActiveAnnouncer(),
        providers: [RepositoryProvider<AnnouncementService>.value(value: mock)],
      );
      await tester.pump(const Duration(seconds: 10));
      await tester.pumpAndSettle();

      verify(mock.announce(expectedAnnouncement));
    });

    testWidgets('ltc7 ltc16 ltc19 No QR code announcement is made repeatedly every 10 seconds', (tester) async {
      final mock = MockAnnouncementService();
      final l10n = await TestUtils.englishLocalizations;
      final expectedAnnouncement = l10n.qrScreenScanScannerActiveWCAGAnnouncement;
      final expectedAnnouncements = 9;

      await tester.pumpWidgetWithAppWrapper(
        const QrScannerActiveAnnouncer(),
        providers: [RepositoryProvider<AnnouncementService>.value(value: mock)],
      );
      await tester.pump(Duration(seconds: expectedAnnouncements * 10 /* 1 announcement every 10 seconds */));
      await tester.pumpAndSettle();

      verify(mock.announce(expectedAnnouncement)).called(expectedAnnouncements);
    });
  });
}
