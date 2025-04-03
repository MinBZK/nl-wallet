import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/page/generic_loading_page.dart';

import '../../../../wallet_app_test_widget.dart';
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
  });
}
