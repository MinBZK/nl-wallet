import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/help/topic_block.dart';
import 'package:wallet/src/util/mapper/help/topic_block_mapper.dart';

void main() {
  late TopicBlockMapper mapper;

  setUp(() => mapper = TopicBlockMapper());

  group('TopicBlockMapper.map', () {
    test('empty input returns empty list', () {
      expect(mapper.map(''), isEmpty);
      expect(mapper.map('   \n\n  '), isEmpty);
    });

    test('single paragraph', () {
      final blocks = mapper.map('NL Wallet is an app for your cards.');
      expect(blocks, [const TopicParagraphBlock('NL Wallet is an app for your cards.')]);
    });

    test('heading-only block', () {
      final blocks = mapper.map('**What must you do?**');
      expect(blocks, [const TopicHeadingBlock('What must you do?')]);
    });

    test('bullet list without preceding heading', () {
      final blocks = mapper.map('- First item.\n- Second item.\n- Third item.');
      expect(blocks, [
        const TopicBulletListBlock(['First item.', 'Second item.', 'Third item.']),
      ]);
    });

    test('heading and bullets are emitted as separate sibling blocks', () {
      const markdown = '**What must you do?**\n\n- Close the app.\n- Open the app again.';
      final blocks = mapper.map(markdown);
      expect(blocks, [
        const TopicHeadingBlock('What must you do?'),
        const TopicBulletListBlock(['Close the app.', 'Open the app again.']),
      ]);
    });

    test('reference block with single link', () {
      final blocks = mapper.map('[What is NL Wallet?](help://what_is_wallet)');
      expect(blocks, [
        const TopicReferenceBlock([
          TopicReferenceLink(label: 'What is NL Wallet?', topicId: 'what_is_wallet'),
        ]),
      ]);
    });

    test('reference block with multiple pipe-separated links', () {
      const markdown = '[A](help://a) | [B](help://b) | [C with spaces](help://c_id)';
      final blocks = mapper.map(markdown);
      expect(blocks, [
        const TopicReferenceBlock([
          TopicReferenceLink(label: 'A', topicId: 'a'),
          TopicReferenceLink(label: 'B', topicId: 'b'),
          TopicReferenceLink(label: 'C with spaces', topicId: 'c_id'),
        ]),
      ]);
    });

    test('full topic body (paragraph + heading + bullets + references)', () {
      const markdown = '''
Start the demo again from the beginning.

**What must you do?**

- Close the app.
- Open the app again.

[Is the NL Wallet demo real?](help://is_demo_real) | [What is NL Wallet?](help://what_is_wallet)
''';
      final blocks = mapper.map(markdown);
      expect(blocks, [
        const TopicParagraphBlock('Start the demo again from the beginning.'),
        const TopicHeadingBlock('What must you do?'),
        const TopicBulletListBlock(['Close the app.', 'Open the app again.']),
        const TopicReferenceBlock([
          TopicReferenceLink(label: 'Is the NL Wallet demo real?', topicId: 'is_demo_real'),
          TopicReferenceLink(label: 'What is NL Wallet?', topicId: 'what_is_wallet'),
        ]),
      ]);
    });

    test('multiple heading+bullets sections in one topic', () {
      const markdown = '''
**How it works**

- Step one.
- Step two.

**Your control**

- Control one.
- Control two.
''';
      final blocks = mapper.map(markdown);
      expect(blocks, [
        const TopicHeadingBlock('How it works'),
        const TopicBulletListBlock(['Step one.', 'Step two.']),
        const TopicHeadingBlock('Your control'),
        const TopicBulletListBlock(['Control one.', 'Control two.']),
      ]);
    });

    test('multi-sentence paragraph on a single line is one paragraph block', () {
      const markdown = 'First sentence. Second sentence. Third sentence.';
      final blocks = mapper.map(markdown);
      expect(blocks, [const TopicParagraphBlock('First sentence. Second sentence. Third sentence.')]);
    });

    test('unrecognised multi-line chunk falls back to paragraph preserving markdown', () {
      // Mix of heading-like line with a non-bullet body — not a shape we emit, but parser
      // must keep the content via the fallback.
      const markdown = '**Inline bold** and some trailing text that is not a bullet list.';
      final blocks = mapper.map(markdown);
      expect(blocks, [const TopicParagraphBlock(markdown)]);
    });

    test('reference block takes precedence over paragraph fallback', () {
      // A single line with only `[...](help://...)` items must be a reference block,
      // not a paragraph — because the paragraph fallback would send it to MarkdownBody
      // which can't distinguish help:// links as structural.
      final blocks = mapper.map('[A](help://a)');
      expect(blocks.single, isA<TopicReferenceBlock>());
    });

    test('single-item bullet list is still a bullet list block', () {
      // 3 topics in the corpus have this shape.
      final blocks = mapper.map('- When your NL Wallet is ready to use.');
      expect(blocks, [
        const TopicBulletListBlock(['When your NL Wallet is ready to use.']),
      ]);
    });
  });
}
