import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/common/widget/activity_summary.dart';
import 'package:wallet/src/util/extension/string_extension.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../util/test_utils.dart';

void main() {
  late AppLocalizations l10n;

  setUp(() async {
    l10n = await TestUtils.englishLocalizations;
  });

  group(
    'mode',
    () {
      test('when no activities are provided, the mode defaults to last month', () {
        const summary = ActivitySummary(attributes: []);
        expect(summary.mode, ActivityDisplayMode.lastMonth);
      });

      test('when all provided activities occurred today, the mode is today', () {
        final summary = ActivitySummary(attributes: [
          WalletMockData.operationTimelineAttribute.copyWith(dateTime: DateTime.now()),
          WalletMockData.operationTimelineAttribute.copyWith(dateTime: DateTime.now()),
        ]);
        expect(summary.mode, ActivityDisplayMode.today);
      });

      test('when the provided activities include activities from the last week, the mode is lastWeek', () {
        final summary = ActivitySummary(attributes: [
          WalletMockData.operationTimelineAttribute.copyWith(dateTime: DateTime.now()),
          WalletMockData.operationTimelineAttribute.copyWith(dateTime: DateTime.now().add(const Duration(days: 3))),
          WalletMockData.operationTimelineAttribute.copyWith(dateTime: DateTime.now().add(const Duration(days: 20))),
        ]);
        expect(summary.mode, ActivityDisplayMode.lastWeek);
      });

      test('when the provided activities only include activities from the more than a week ago, the mode is lastMonth',
          () {
        final summary = ActivitySummary(attributes: [
          WalletMockData.operationTimelineAttribute.copyWith(dateTime: DateTime.now().add(const Duration(days: 8))),
          WalletMockData.operationTimelineAttribute.copyWith(dateTime: DateTime.now().add(const Duration(days: 20))),
        ]);
        expect(summary.mode, ActivityDisplayMode.lastWeek);
      });
    },
  );

  group('widgets', () {
    testWidgets('empty state shows no activities', (tester) async {
      await tester.pumpWidgetWithAppWrapper(const ActivitySummary(attributes: []));

      final emptyFinder = find.text(l10n.activitySummaryEmpty);
      expect(emptyFinder, findsOneWidget);
    });

    testWidgets('card added subtitle is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        ActivitySummary(
          attributes: [
            WalletMockData.operationTimelineAttribute.copyWith(dateTime: DateTime.now()),
          ],
        ),
      );

      final cardsAddedFinder = find.textContaining(
        l10n.activitySummaryCardsAdded(1, 1),
      );
      expect(cardsAddedFinder, findsOneWidget);
    });

    testWidgets('multiple cards added subtitle is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        ActivitySummary(
          attributes: [
            WalletMockData.operationTimelineAttribute.copyWith(dateTime: DateTime.now()),
            WalletMockData.operationTimelineAttribute.copyWith(dateTime: DateTime.now()),
            WalletMockData.operationTimelineAttribute.copyWith(dateTime: DateTime.now()),
          ],
        ),
      );

      final cardsAddedFinder = find.textContaining(
        l10n.activitySummaryCardsAdded(3, 3),
      );
      expect(cardsAddedFinder, findsOneWidget);
    });

    testWidgets('relevant organization name is shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        ActivitySummary(
          attributes: [
            WalletMockData.interactionTimelineAttribute.copyWith(dateTime: DateTime.now()),
          ],
        ),
      );

      final organizationFinder = find.textContaining(
        l10n.activitySummarySharedWith(WalletMockData.interactionTimelineAttribute.organization.displayName.testValue),
      );
      expect(organizationFinder, findsOneWidget);
    });

    testWidgets('relevant organization name is only shown once (no duplicates)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        ActivitySummary(
          attributes: [
            WalletMockData.interactionTimelineAttribute.copyWith(dateTime: DateTime.now()),
            WalletMockData.interactionTimelineAttribute.copyWith(dateTime: DateTime.now()),
            WalletMockData.interactionTimelineAttribute.copyWith(dateTime: DateTime.now()),
          ],
        ),
      );

      final organizationFinder = find.textContaining(
        l10n.activitySummarySharedWith(WalletMockData.interactionTimelineAttribute.organization.displayName.testValue),
      );
      expect(organizationFinder, findsOneWidget);
    });

    testWidgets('all 3 relevant organization names are all shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        ActivitySummary(
          attributes: [
            WalletMockData.interactionTimelineAttribute.copyWith(
              dateTime: DateTime.now(),
              organization: WalletMockData.organization.copyWith(
                displayName: 'Org-X'.untranslated,
              ),
            ),
            WalletMockData.interactionTimelineAttribute.copyWith(
              dateTime: DateTime.now(),
              organization: WalletMockData.organization.copyWith(
                displayName: 'Org-Y'.untranslated,
              ),
            ),
            WalletMockData.interactionTimelineAttribute.copyWith(
              dateTime: DateTime.now(),
              organization: WalletMockData.organization.copyWith(
                displayName: 'Org-Z'.untranslated,
              ),
            ),
          ],
        ),
      );

      final orgXFinder = find.textContaining('Org-X');
      final orgYFinder = find.textContaining('Org-Y');
      final orgZFinder = find.textContaining('Org-Z');

      expect(orgXFinder, findsOneWidget);
      expect(orgYFinder, findsOneWidget);
      expect(orgZFinder, findsOneWidget);
    });

    testWidgets('when there >3 relevant organizations, only write out the first two', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        ActivitySummary(
          attributes: [
            WalletMockData.interactionTimelineAttribute.copyWith(
              dateTime: DateTime.now(),
              organization: WalletMockData.organization.copyWith(
                displayName: 'Org-X'.untranslated,
              ),
            ),
            WalletMockData.interactionTimelineAttribute.copyWith(
              dateTime: DateTime.now(),
              organization: WalletMockData.organization.copyWith(
                displayName: 'Org-Y'.untranslated,
              ),
            ),
            WalletMockData.interactionTimelineAttribute.copyWith(
              dateTime: DateTime.now(),
              organization: WalletMockData.organization.copyWith(
                displayName: 'not-shown-a'.untranslated,
              ),
            ),
            WalletMockData.interactionTimelineAttribute.copyWith(
              dateTime: DateTime.now(),
              organization: WalletMockData.organization.copyWith(
                displayName: 'not-shown-b'.untranslated,
              ),
            ),
          ],
        ),
      );

      final orgXFinder = find.textContaining('Org-X');
      final orgYFinder = find.textContaining('Org-Y');
      final notShownA = find.textContaining('not-shown-a');
      final notShownB = find.textContaining('not-shown-b');

      expect(orgXFinder, findsOneWidget);
      expect(orgYFinder, findsOneWidget);
      expect(notShownA, findsNothing);
      expect(notShownB, findsNothing);

      /// Validate that not shown widgets are shown as group.
      final otherOrgsFinder = find.textContaining(l10n.activitySummarySharedWithOthers(2));
      expect(otherOrgsFinder, findsOneWidget);
    });
  });
}
