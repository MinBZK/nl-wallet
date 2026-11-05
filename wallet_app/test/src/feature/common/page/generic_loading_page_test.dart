import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/announcement_service.dart';
import 'package:wallet/src/feature/common/page/generic_loading_page.dart';
import 'package:wallet/src/wallet_assets.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('generic loading light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        GenericLoadingPage(
          title: 'Title',
          description: 'Description',
          onCancel: () {},
          cancelCta: 'Cancel',
        ),
      );
      await screenMatchesGolden('generic_loading_page/light');
    });

    testGoldens('generic loading light - with context logo', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        GenericLoadingPage(
          title: 'Title',
          description: 'Description',
          onCancel: () {},
          contextImage: Image.asset(WalletAssets.logo_wallet, height: 64, width: 64),
          cancelCta: 'Cancel',
        ),
      );
      await screenMatchesGolden('generic_loading_page/context_image.light');
    });

    testGoldens('generic loading dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        GenericLoadingPage(
          title: 'Title',
          description: 'Description',
          onCancel: () {},
          cancelCta: 'Cancel',
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('generic_loading_page/dark');
    });
  });

  group('widgets', () {
    testWidgets('GenericLoadingPage renders expected widgets without cancelButton', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const WalletAppTestWidget(
          child: GenericLoadingPage(
            title: 'T',
            description: 'D',
            cancelCta: 'C',
          ),
        ),
      );

      // Setup finders
      final titleFinder = find.text('T');
      final descriptionFinder = find.text('D');
      final cancelButtonFinder = find.text('C');

      // Verify all expected widgets show up once
      expect(titleFinder, findsOneWidget);
      expect(descriptionFinder, findsOneWidget);
      expect(cancelButtonFinder, findsNothing);
    });

    testWidgets('GenericLoadingPage renders expected widgets with cancelButton', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        WalletAppTestWidget(
          child: GenericLoadingPage(
            title: 'T',
            description: 'D',
            cancelCta: 'C',
            onCancel: () {},
          ),
        ),
      );

      // Setup finders
      final titleFinder = find.text('T');
      final descriptionFinder = find.text('D');
      final cancelButtonFinder = find.text('C');

      // Verify all expected widgets show up once
      expect(titleFinder, findsOneWidget);
      expect(descriptionFinder, findsOneWidget);
      expect(cancelButtonFinder, findsOneWidget);
    });

    testWidgets('when cancel button is pressed the onCancel callback is triggered', (tester) async {
      bool isCalled = false;
      await tester.pumpWidgetWithAppWrapper(
        WalletAppTestWidget(
          child: GenericLoadingPage(
            title: 'T',
            description: 'D',
            cancelCta: 'C',
            onCancel: () => isCalled = true,
          ),
        ),
      );

      final cancelButtonFinder = find.text('C');
      await tester.tap(cancelButtonFinder);
      expect(isCalled, isTrue);
    });

    testWidgets('GenericLoadingPage without description only exposes the title', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const GenericLoadingPage(title: 'T'),
      );

      expect(find.text('T'), findsOneWidget);
      expect(
        tester.getSemantics(find.text('T')),
        matchesSemantics(label: 'T', isHeader: true),
      );
      expect(find.byType(Text), findsOneWidget);
    });

    group('title announcements', () {
      late MockAnnouncementService announcementService;
      late String title;
      late StateSetter setTitleState;

      setUp(() {
        announcementService = MockAnnouncementService();
        title = 'Please wait';
      });

      Future<void> updateTitle(WidgetTester tester, String newTitle) async {
        setTitleState(() => title = newTitle);
        await tester.pump();
      }

      Future<void> pumpPage(WidgetTester tester, {bool requestAccessibilityFocus = true}) {
        return tester.pumpWidgetWithAppWrapper(
          StatefulBuilder(
            builder: (context, setState) {
              setTitleState = setState;
              return GenericLoadingPage(
                title: title,
                requestAccessibilityFocus: requestAccessibilityFocus,
              );
            },
          ),
          providers: [RepositoryProvider<AnnouncementService>.value(value: announcementService)],
        );
      }

      testWidgets('title is not announced on mount, the focus request covers that', (tester) async {
        await pumpPage(tester);

        verifyNever(announcementService.announce(any));
      });

      testWidgets('title that changes in place is announced once', (tester) async {
        await pumpPage(tester);

        await updateTitle(tester, 'Continue in your DigiD app');
        // A rebuild with the same title must not repeat the announcement.
        await updateTitle(tester, 'Continue in your DigiD app');

        verify(announcementService.announce('Continue in your DigiD app')).called(1);
      });

      testWidgets('title change is not announced when the page manages its own focus', (tester) async {
        await pumpPage(tester, requestAccessibilityFocus: false);

        await updateTitle(tester, 'Continue in your DigiD app');

        verifyNever(announcementService.announce(any));
      });
    });
  });
}
