import '../../../domain/model/help/topic_block.dart';
import '../mapper.dart';

/// Parses a topic's body markdown into a list of [TopicBlock]s.
///
/// Input follows the conventions produced by the content importer: blocks are
/// separated by one or more blank lines, and each block has one of a small set
/// of recognised shapes.
///
/// | Shape                                  | Block                   |
/// |----------------------------------------|-------------------------|
/// | `[label](help://id) \| [label](help://id) …` | [TopicReferenceBlock]   |
/// | `**Subheading**`                       | [TopicHeadingBlock]     |
/// | `- item` (every line starts with `- `) | [TopicBulletListBlock]  |
/// | anything else                          | [TopicParagraphBlock]   |
///
/// The paragraph shape is the fallback — it absorbs any chunk that doesn't
/// match the shapes above, so inline markdown still renders.
class TopicBlockMapper extends Mapper<String, List<TopicBlock>> {
  static const _bulletPrefix = '- ';

  /// Whole line is one or more `[label](help://id)` items joined by ` | `. Any
  /// extra prose on the line disqualifies the match and the chunk falls through
  /// to the paragraph handler.
  ///
  /// Shape: `[…](help://…) ( \| […](help://…) )*`
  static final _referenceLineShape = RegExp(
    r'^\s*\[[^\]]+\]\(help://[^)]+\)(\s*\|\s*\[[^\]]+\]\(help://[^)]+\))*\s*$',
  );

  /// Captures a single `[label](help://id)` occurrence; run against a line
  /// already known to be a reference line to extract each link.
  static final _linkToken = RegExp(r'\[([^\]]+)\]\(help://([^)]+)\)');

  /// Whole line is exactly `**text**`. Inline bold inside a paragraph does not
  /// match because the pattern is anchored to start/end of line.
  static final _headingLineShape = RegExp(r'^\*\*(.+)\*\*$');

  @override
  List<TopicBlock> map(String input) {
    final chunks = _splitIntoChunks(input);
    return chunks.map(_classify).whereType<TopicBlock>().toList();
  }

  /// Splits the markdown into non-empty groups of consecutive non-blank lines.
  /// Blank lines are pure separators and are dropped.
  List<List<String>> _splitIntoChunks(String markdown) {
    final chunks = <List<String>>[];
    var buffer = <String>[];
    for (final line in markdown.split('\n')) {
      if (line.trim().isEmpty) {
        if (buffer.isNotEmpty) {
          chunks.add(buffer);
          buffer = <String>[];
        }
      } else {
        buffer.add(line);
      }
    }
    if (buffer.isNotEmpty) chunks.add(buffer);
    return chunks;
  }

  /// Classifies a non-empty chunk into a single block. Tries each recognised
  /// shape in order; the paragraph handler is always the last resort so every
  /// non-empty chunk produces a block.
  ///
  /// Order matters: reference and heading are single-line shapes that look like
  /// plain text to the paragraph handler, so they must be tested first.
  TopicBlock? _classify(List<String> lines) {
    if (lines.isEmpty) return null;

    final singleLineBlock = _tryParseAsSingleLineBlock(lines);
    if (singleLineBlock != null) return singleLineBlock;

    final bulletList = _tryParseAsBulletList(lines);
    if (bulletList != null) return bulletList;

    return _parseAsParagraph(lines);
  }

  /// Attempts to read a single-line chunk as either a [TopicReferenceBlock] or
  /// a [TopicHeadingBlock]. Returns null for multi-line chunks or when neither
  /// shape matches.
  TopicBlock? _tryParseAsSingleLineBlock(List<String> lines) {
    if (lines.length != 1) return null;
    final line = lines.first.trim();

    if (_referenceLineShape.hasMatch(line)) {
      return _parseReferenceBlock(line);
    }

    final headingMatch = _headingLineShape.firstMatch(line);
    if (headingMatch != null) {
      return TopicHeadingBlock(headingMatch.group(1)!);
    }

    return null;
  }

  /// Extracts every `[label](help://id)` on the line into a [TopicReferenceLink].
  /// Caller guarantees the whole-line shape via [_referenceLineShape].
  TopicReferenceBlock _parseReferenceBlock(String line) {
    final links = _linkToken.allMatches(line).map((match) {
      return TopicReferenceLink(
        label: match.group(1)!,
        topicId: match.group(2)!,
      );
    }).toList();
    return TopicReferenceBlock(links);
  }

  /// A bullet list requires *every* line in the chunk to start with `- `. One
  /// stray non-bullet line disqualifies the whole chunk (it then falls through
  /// to the paragraph handler).
  TopicBulletListBlock? _tryParseAsBulletList(List<String> lines) {
    if (!lines.every(_isBulletLine)) return null;
    final items = lines.map(_stripBulletPrefix).toList();
    return TopicBulletListBlock(items);
  }

  bool _isBulletLine(String line) => line.trimLeft().startsWith(_bulletPrefix);

  String _stripBulletPrefix(String line) => line.trimLeft().substring(_bulletPrefix.length).trimRight();

  /// Fallback for any chunk not recognised as a reference, heading, or bullet
  /// list. The original newline layout is preserved so inline markdown renders
  /// as authored.
  TopicParagraphBlock _parseAsParagraph(List<String> lines) => TopicParagraphBlock(lines.join('\n'));
}
