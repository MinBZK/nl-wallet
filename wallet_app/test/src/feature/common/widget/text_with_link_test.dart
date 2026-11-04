import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/text_with_link.dart';

import '../../../../wallet_app_test_widget.dart';

void main() {
  group('widgets', () {
    testWidgets('text and link are exposed as separate nodes in reading order', (tester) async {
      final semanticsHandle = tester.ensureSemantics();
      await tester.pumpWidgetWithAppWrapper(
        TextWithLink(
          fullText: 'Read more on example.org today.',
          linkText: 'example.org',
          linkUrl: Uri.parse('https://example.org'),
          onLinkPressed: () {},
        ),
      );

      final link = find.semantics.byLabel('example.org').evaluate().single;
      expect(link, isSemantics(isLink: true, isButton: false, hasTapAction: true, isFocusable: true));
      expect(link.getSemanticsData().linkUrl, Uri.parse('https://example.org'));

      final leadingText = find.semantics.byLabel('Read more on ').evaluate().single;
      final trailingText = find.semantics.byLabel(' today.').evaluate().single;
      expect(leadingText, isSemantics(isLink: false, hasTapAction: false));
      expect(trailingText, isSemantics(isLink: false, hasTapAction: false));

      final readingOrder = leadingText.parent!.debugListChildrenInOrder(DebugSemanticsDumpOrder.traversalOrder);
      expect(readingOrder, [leadingText, link, trailingText]);
      semanticsHandle.dispose();
    });

    testWidgets('tapping the link triggers onLinkPressed', (tester) async {
      bool isPressed = false;
      await tester.pumpWidgetWithAppWrapper(
        TextWithLink(
          fullText: 'Read more on example.org today.',
          linkText: 'example.org',
          onLinkPressed: () => isPressed = true,
        ),
      );

      await tester.tap(find.text('example.org', findRichText: true));
      expect(isPressed, isTrue);
    });
  });
}
