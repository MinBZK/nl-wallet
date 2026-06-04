import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/util/help_content_validator.dart';

/// Baseline help corpus — valid; produces zero issues. Individual tests tweak
/// one aspect (missing file, broken link, missing translation, …) to assert
/// each rule in isolation.
const _kBaselineYaml = '''
structure:
  - categoryId: cat1
    icon: qr_code
    subcategories:
      - subcategoryId: sub1
        topics:
          - groupId: help
            topicIds:
              - topic_a
              - topic_b
translations:
  en:
    categories:
      cat1: {title: Cat1, subtitle: Subtitle}
    subcategories:
      sub1: Sub1
    topics:
      topic_a: Topic A
      topic_b: Topic B
  nl:
    categories:
      cat1: {title: Cat1 NL, subtitle: Ondertitel}
    subcategories:
      sub1: Sub1 NL
    topics:
      topic_a: Topic A NL
      topic_b: Topic B NL
''';

/// Creates a temp dir and writes a baseline valid help corpus into it.
/// Individual tests mutate files via [writeFile] / [deleteFile] to set up the
/// scenario under test.
class _TempCorpus {
  final Directory root;

  _TempCorpus._(this.root);

  factory _TempCorpus.create({String yaml = _kBaselineYaml}) {
    final root = Directory.systemTemp.createTempSync('help_validator_');
    Directory('${root.path}/en').createSync();
    Directory('${root.path}/nl').createSync();
    File('${root.path}/help.yaml').writeAsStringSync(yaml);
    File('${root.path}/en/topic_a.md').writeAsStringSync('body A');
    File('${root.path}/en/topic_b.md').writeAsStringSync('body B');
    File('${root.path}/nl/topic_a.md').writeAsStringSync('inhoud A');
    File('${root.path}/nl/topic_b.md').writeAsStringSync('inhoud B');
    return _TempCorpus._(root);
  }

  String get path => root.path;

  void writeFile(String relativePath, String content) => File('${root.path}/$relativePath').writeAsStringSync(content);

  void deleteFile(String relativePath) => File('${root.path}/$relativePath').deleteSync();

  void dispose() => root.deleteSync(recursive: true);
}

void main() {
  late _TempCorpus corpus;

  setUp(() => corpus = _TempCorpus.create());
  tearDown(() => corpus.dispose());

  test('baseline corpus produces zero issues', () {
    final result = validateHelpContent(helpDir: corpus.path);
    expect(result.issues, isEmpty);
    expect(result.hasIssues, isFalse);
  });

  group('errors — missing markdown', () {
    test('missing EN markdown produces error against help.yaml', () {
      corpus.deleteFile('en/topic_a.md');
      final result = validateHelpContent(helpDir: corpus.path);
      expect(result.issues, hasLength(1));
      final error = result.issues.single;
      expect(error.file, 'help.yaml');
      expect(error.message, contains("'topic_a'"));
      expect(error.message, contains('en/topic_a.md'));
    });

    test('missing NL markdown produces error', () {
      corpus.deleteFile('nl/topic_b.md');
      final result = validateHelpContent(helpDir: corpus.path);
      expect(result.issues.single.message, contains('nl/topic_b.md'));
    });

    test('missing in both locales produces two errors', () {
      corpus
        ..deleteFile('en/topic_a.md')
        ..deleteFile('nl/topic_a.md');
      final result = validateHelpContent(helpDir: corpus.path);
      expect(result.issues, hasLength(2));
    });
  });

  group('errors — orphan markdown', () {
    test('markdown file without YAML entry is flagged', () {
      corpus.writeFile('en/stranger.md', 'i do not belong');
      final result = validateHelpContent(helpDir: corpus.path);
      final orphan = result.issues.where((e) => e.file == 'en/stranger.md').single;
      expect(orphan.message, contains('orphan'));
      expect(orphan.message, contains("'stranger'"));
    });

    test('orphan in NL is flagged independently of EN', () {
      corpus.writeFile('nl/lost.md', 'x');
      final result = validateHelpContent(helpDir: corpus.path);
      expect(result.issues.where((e) => e.file == 'nl/lost.md'), hasLength(1));
    });

    test('non-markdown files in a locale directory are ignored', () {
      corpus.writeFile('en/notes.txt', 'not markdown');
      final result = validateHelpContent(helpDir: corpus.path);
      expect(result.issues, isEmpty);
    });
  });

  group('errors — broken help:// links', () {
    test('link to unknown topicId produces error', () {
      corpus.writeFile('en/topic_a.md', '[dangling](help://does_not_exist)');
      final result = validateHelpContent(helpDir: corpus.path);
      final error = result.issues.single;
      expect(error.file, 'en/topic_a.md');
      expect(error.message, contains('help://does_not_exist'));
      expect(error.message, contains('unknown topicId'));
    });

    test('link to a known topicId produces no error', () {
      corpus.writeFile('en/topic_a.md', '[topic b](help://topic_b)');
      final result = validateHelpContent(helpDir: corpus.path);
      expect(result.issues, isEmpty);
    });

    test('broken link in NL is flagged independently of EN', () {
      corpus.writeFile('nl/topic_b.md', '[kapot](help://unknown)');
      final result = validateHelpContent(helpDir: corpus.path);
      expect(result.issues.single.file, 'nl/topic_b.md');
    });

    test('multiple broken links in one file produce multiple errors', () {
      corpus.writeFile(
        'en/topic_a.md',
        '[a](help://missing_one) | [b](help://missing_two)',
      );
      final result = validateHelpContent(helpDir: corpus.path);
      expect(result.issues, hasLength(2));
    });

    test('help:// links inside a paragraph (non-reference shape) are not scanned', () {
      // Inline links live inside a paragraph chunk; the mapper produces a
      // TopicParagraphBlock for it, so the validator does not walk them. This
      // documents the current behaviour — see IMPORT_STATUS notes on inline
      // links being absent from the corpus.
      corpus.writeFile('en/topic_a.md', 'prose with inline [link](help://unknown) inside');
      final result = validateHelpContent(helpDir: corpus.path);
      expect(result.issues, isEmpty);
    });
  });

  group('errors — translations', () {
    test('missing translations block is a single error', () {
      const yaml = '''
structure:
  - categoryId: cat1
    icon: qr_code
    subcategories:
      - subcategoryId: sub1
        topics:
          - groupId: help
            topicIds:
              - topic_a
''';
      final c = _TempCorpus.create(yaml: yaml);
      try {
        final result = validateHelpContent(helpDir: c.path);
        expect(
          result.issues.where((e) => e.message.contains('translations block')),
          isNotEmpty,
        );
      } finally {
        c.dispose();
      }
    });

    test('missing locale block is flagged', () {
      const yaml = '''
structure:
  - categoryId: cat1
    icon: qr_code
    subcategories:
      - subcategoryId: sub1
        topics:
          - groupId: help
            topicIds:
              - topic_a
translations:
  en:
    categories:
      cat1: {title: Cat1, subtitle: Subtitle}
    subcategories:
      sub1: Sub1
    topics:
      topic_a: Topic A
''';
      final c = _TempCorpus.create(yaml: yaml);
      try {
        final result = validateHelpContent(helpDir: c.path);
        expect(
          result.issues.where((e) => e.message.contains('translations.nl')),
          hasLength(1),
        );
      } finally {
        c.dispose();
      }
    });

    test('missing category translation is flagged per locale', () {
      const yaml = '''
structure:
  - categoryId: cat1
    icon: qr_code
    subcategories:
      - subcategoryId: sub1
        topics:
          - groupId: help
            topicIds:
              - topic_a
translations:
  en:
    categories: {}
    subcategories:
      sub1: Sub1
    topics:
      topic_a: Topic A
  nl:
    categories: {}
    subcategories:
      sub1: Sub1 NL
    topics:
      topic_a: Topic A NL
''';
      final c = _TempCorpus.create(yaml: yaml);
      try {
        final result = validateHelpContent(helpDir: c.path);
        final categoryErrors = result.issues.where((e) => e.message.contains("category 'cat1'"));
        expect(categoryErrors, hasLength(2), reason: 'one for each locale');
      } finally {
        c.dispose();
      }
    });

    test('category missing title is flagged even when entry exists', () {
      const yaml = '''
structure:
  - categoryId: cat1
    icon: qr_code
    subcategories:
      - subcategoryId: sub1
        topics:
          - groupId: help
            topicIds:
              - topic_a
translations:
  en:
    categories:
      cat1: {subtitle: Only subtitle}
    subcategories:
      sub1: Sub1
    topics:
      topic_a: Topic A
  nl:
    categories:
      cat1: {title: Title only}
    subcategories:
      sub1: Sub1 NL
    topics:
      topic_a: Topic A NL
''';
      final c = _TempCorpus.create(yaml: yaml);
      try {
        final result = validateHelpContent(helpDir: c.path);
        expect(
          result.issues.map((e) => e.message),
          containsAll(<Matcher>[
            contains('en.title'),
            contains('nl.subtitle'),
          ]),
        );
      } finally {
        c.dispose();
      }
    });

    test('missing subcategory translation is flagged', () {
      const yaml = '''
structure:
  - categoryId: cat1
    icon: qr_code
    subcategories:
      - subcategoryId: sub1
        topics:
          - groupId: help
            topicIds:
              - topic_a
translations:
  en:
    categories:
      cat1: {title: Cat1, subtitle: Subtitle}
    subcategories: {}
    topics:
      topic_a: Topic A
  nl:
    categories:
      cat1: {title: Cat1 NL, subtitle: Ondertitel}
    subcategories: {}
    topics:
      topic_a: Topic A NL
''';
      final c = _TempCorpus.create(yaml: yaml);
      try {
        final result = validateHelpContent(helpDir: c.path);
        final subErrors = result.issues.where((e) => e.message.contains("subcategory 'sub1'"));
        expect(subErrors, hasLength(2));
      } finally {
        c.dispose();
      }
    });

    test('blank topic translation is flagged as missing/blank', () {
      const yaml = '''
structure:
  - categoryId: cat1
    icon: qr_code
    subcategories:
      - subcategoryId: sub1
        topics:
          - groupId: help
            topicIds:
              - topic_a
translations:
  en:
    categories:
      cat1: {title: Cat1, subtitle: Subtitle}
    subcategories:
      sub1: Sub1
    topics:
      topic_a: ""
  nl:
    categories:
      cat1: {title: Cat1 NL, subtitle: Ondertitel}
    subcategories:
      sub1: Sub1 NL
    topics:
      topic_a: "   "
''';
      final c = _TempCorpus.create(yaml: yaml);
      try {
        final result = validateHelpContent(helpDir: c.path);
        final topicErrors = result.issues.where((e) => e.message.contains("topic 'topic_a'"));
        expect(topicErrors, hasLength(2), reason: 'one for each locale (empty + whitespace-only)');
        expect(topicErrors.every((e) => e.message.contains('missing/blank')), isTrue);
      } finally {
        c.dispose();
      }
    });

    test('blank category title and subcategory are flagged as missing/blank', () {
      const yaml = '''
structure:
  - categoryId: cat1
    icon: qr_code
    subcategories:
      - subcategoryId: sub1
        topics:
          - groupId: help
            topicIds:
              - topic_a
translations:
  en:
    categories:
      cat1: {title: "", subtitle: Subtitle}
    subcategories:
      sub1: ""
    topics:
      topic_a: Topic A
  nl:
    categories:
      cat1: {title: Cat1 NL, subtitle: Ondertitel}
    subcategories:
      sub1: Sub1 NL
    topics:
      topic_a: Topic A NL
''';
      final c = _TempCorpus.create(yaml: yaml);
      try {
        final result = validateHelpContent(helpDir: c.path);
        expect(
          result.issues.map((e) => e.message),
          containsAll(<Matcher>[
            allOf(contains("category 'cat1'"), contains('missing/blank en.title')),
            allOf(contains("subcategory 'sub1'"), contains('missing/blank en')),
          ]),
        );
      } finally {
        c.dispose();
      }
    });

    test('missing topic translation is flagged', () {
      const yaml = '''
structure:
  - categoryId: cat1
    icon: qr_code
    subcategories:
      - subcategoryId: sub1
        topics:
          - groupId: help
            topicIds:
              - topic_a
translations:
  en:
    categories:
      cat1: {title: Cat1, subtitle: Subtitle}
    subcategories:
      sub1: Sub1
    topics: {}
  nl:
    categories:
      cat1: {title: Cat1 NL, subtitle: Ondertitel}
    subcategories:
      sub1: Sub1 NL
    topics: {}
''';
      final c = _TempCorpus.create(yaml: yaml);
      try {
        final result = validateHelpContent(helpDir: c.path);
        final topicErrors = result.issues.where((e) => e.message.contains("topic 'topic_a'"));
        expect(topicErrors, hasLength(2));
      } finally {
        c.dispose();
      }
    });
  });

  group('errors — category icons', () {
    test('unknown icon name is flagged per category', () {
      const yaml = '''
structure:
  - categoryId: cat1
    icon: not_a_real_icon
    subcategories:
      - subcategoryId: sub1
        topics:
          - groupId: help
            topicIds:
              - topic_a
translations:
  en:
    categories:
      cat1: {title: Cat1, subtitle: Subtitle}
    subcategories:
      sub1: Sub1
    topics:
      topic_a: Topic A
  nl:
    categories:
      cat1: {title: Cat1 NL, subtitle: Ondertitel}
    subcategories:
      sub1: Sub1 NL
    topics:
      topic_a: Topic A NL
''';
      final c = _TempCorpus.create(yaml: yaml);
      try {
        final result = validateHelpContent(helpDir: c.path);
        final iconError = result.issues.where((e) => e.message.contains("unknown icon 'not_a_real_icon'")).single;
        expect(iconError.file, 'help.yaml');
        expect(iconError.message, contains("category 'cat1'"));
      } finally {
        c.dispose();
      }
    });

    test('missing icon field is flagged', () {
      // Manually crafted YAML where the icon key is absent (not just empty).
      const yaml = '''
structure:
  - categoryId: cat1
    subcategories:
      - subcategoryId: sub1
        topics:
          - groupId: help
            topicIds:
              - topic_a
translations:
  en:
    categories:
      cat1: {title: Cat1, subtitle: Subtitle}
    subcategories:
      sub1: Sub1
    topics:
      topic_a: Topic A
  nl:
    categories:
      cat1: {title: Cat1 NL, subtitle: Ondertitel}
    subcategories:
      sub1: Sub1 NL
    topics:
      topic_a: Topic A NL
''';
      final c = _TempCorpus.create(yaml: yaml);
      try {
        final result = validateHelpContent(helpDir: c.path);
        final noIcon = result.issues.where((e) => e.message.contains("category 'cat1' has no icon")).single;
        expect(noIcon.file, 'help.yaml');
      } finally {
        c.dispose();
      }
    });

    test('each of the 8 supported icons is accepted', () {
      // Baseline uses qr_code; swap in every supported name and assert no error.
      for (final name in const [
        'credit_card',
        'history',
        'lock_outline',
        'mobile_screen_share',
        'move_up',
        'qr_code',
        'settings',
        'start',
      ]) {
        final yaml =
            '''
structure:
  - categoryId: cat1
    icon: $name
    subcategories:
      - subcategoryId: sub1
        topics:
          - groupId: help
            topicIds:
              - topic_a
translations:
  en:
    categories:
      cat1: {title: Cat1, subtitle: Subtitle}
    subcategories:
      sub1: Sub1
    topics:
      topic_a: Topic A
  nl:
    categories:
      cat1: {title: Cat1 NL, subtitle: Ondertitel}
    subcategories:
      sub1: Sub1 NL
    topics:
      topic_a: Topic A NL
''';
        final c = _TempCorpus.create(yaml: yaml);
        try {
          final result = validateHelpContent(helpDir: c.path);
          expect(
            result.issues.where((e) => e.message.contains('icon')),
            isEmpty,
            reason: 'icon name "$name" should be accepted',
          );
        } finally {
          c.dispose();
        }
      }
    });
  });

  group('errors — group ids', () {
    test('unknown groupId is flagged against help.yaml', () {
      const yaml = '''
structure:
  - categoryId: cat1
    icon: qr_code
    subcategories:
      - subcategoryId: sub1
        topics:
          - groupId: nonsense
            topicIds:
              - topic_a
translations:
  en:
    categories:
      cat1: {title: Cat1, subtitle: Subtitle}
    subcategories:
      sub1: Sub1
    topics:
      topic_a: Topic A
  nl:
    categories:
      cat1: {title: Cat1 NL, subtitle: Ondertitel}
    subcategories:
      sub1: Sub1 NL
    topics:
      topic_a: Topic A NL
''';
      final c = _TempCorpus.create(yaml: yaml);
      try {
        final result = validateHelpContent(helpDir: c.path);
        final groupError = result.issues.where((e) => e.message.contains("groupId 'nonsense'")).single;
        expect(groupError.file, 'help.yaml');
        expect(groupError.message, contains("subcategory 'sub1'"));
      } finally {
        c.dispose();
      }
    });

    test("the 'information' group id is accepted", () {
      const yaml = '''
structure:
  - categoryId: cat1
    icon: qr_code
    subcategories:
      - subcategoryId: sub1
        topics:
          - groupId: information
            topicIds:
              - topic_a
translations:
  en:
    categories:
      cat1: {title: Cat1, subtitle: Subtitle}
    subcategories:
      sub1: Sub1
    topics:
      topic_a: Topic A
  nl:
    categories:
      cat1: {title: Cat1 NL, subtitle: Ondertitel}
    subcategories:
      sub1: Sub1 NL
    topics:
      topic_a: Topic A NL
''';
      final c = _TempCorpus.create(yaml: yaml);
      try {
        final result = validateHelpContent(helpDir: c.path);
        expect(result.issues.where((e) => e.message.contains('groupId')), isEmpty);
      } finally {
        c.dispose();
      }
    });
  });

  group('ValidationResult', () {
    test('hasIssues is false on a clean corpus', () {
      final result = validateHelpContent(helpDir: corpus.path);
      expect(result.hasIssues, isFalse);
    });

    test('hasIssues is true when any issue present', () {
      corpus.writeFile('en/topic_a.md', '[x](help://unknown)');
      final result = validateHelpContent(helpDir: corpus.path);
      expect(result.hasIssues, isTrue);
    });
  });
}
