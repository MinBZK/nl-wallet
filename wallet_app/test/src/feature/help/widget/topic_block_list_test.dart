import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:wallet/src/domain/model/help/topic_block.dart';
import 'package:wallet/src/feature/help/widget/topic_block_list.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/test_utils.dart';

void main() {
  group('TopicBlockList rendering', () {
    testWidgets('renders heading block as a Semantics header', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TopicBlockList(blocks: [TopicHeadingBlock('What must you do?')]),
      );
      expect(find.text('What must you do?'), findsOneWidget);
      expect(
        tester.getSemantics(find.text('What must you do?')),
        matchesSemantics(isHeader: true, label: 'What must you do?'),
      );
    });

    testWidgets('renders paragraph block via MarkdownBody', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TopicBlockList(blocks: [TopicParagraphBlock('Start the demo again.')]),
      );
      expect(find.textContaining('Start the demo again.'), findsOneWidget);
    });

    testWidgets('renders each bullet list item as natural text', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TopicBlockList(
          blocks: [
            TopicBulletListBlock(['Close the app.', 'Open the app again.']),
          ],
        ),
      );
      expect(find.textContaining('Close the app.'), findsOneWidget);
      expect(find.textContaining('Open the app again.'), findsOneWidget);
    });

    testWidgets('renders reference block with localized heading', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TopicBlockList(
          blocks: [
            TopicReferenceBlock([
              TopicReferenceLink(label: 'Is the demo real?', topicId: 'is_demo_real'),
            ]),
          ],
        ),
      );
      final AppLocalizations l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.helpTopicScreenReferencesHeading), findsOneWidget);
      expect(find.text('Is the demo real?'), findsOneWidget);
    });
  });

  group('TopicBlockList reference tap', () {
    testWidgets('invokes onReferenceTap with the target topicId', (tester) async {
      String? tappedTopicId;
      await tester.pumpWidgetWithAppWrapper(
        TopicBlockList(
          blocks: const [
            TopicReferenceBlock([
              TopicReferenceLink(label: 'Target topic', topicId: 'target_cid'),
            ]),
          ],
          onReferenceTap: (topicId) => tappedTopicId = topicId,
        ),
      );
      await tester.tap(find.text('Target topic'));
      expect(tappedTopicId, 'target_cid');
    });

    testWidgets('does not crash when onReferenceTap is null', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TopicBlockList(
          blocks: [
            TopicReferenceBlock([
              TopicReferenceLink(label: 'Target', topicId: 'target'),
            ]),
          ],
        ),
      );
      await tester.tap(find.text('Target'));
      // No expectation — the widget must simply stay alive.
    });

    testWidgets('inline help:// link in a paragraph also invokes onReferenceTap', (tester) async {
      String? tappedTopicId;
      await tester.pumpWidgetWithAppWrapper(
        TopicBlockList(
          blocks: const [
            TopicParagraphBlock('See [this topic](help://inline_target) for details.'),
          ],
          onReferenceTap: (topicId) => tappedTopicId = topicId,
        ),
      );
      await tester.tap(find.textContaining('this topic'));
      expect(tappedTopicId, 'inline_target');
    });
  });
}
