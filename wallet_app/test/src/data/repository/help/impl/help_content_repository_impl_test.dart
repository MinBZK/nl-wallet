import 'dart:convert';
import 'dart:ui';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/data/repository/help/impl/help_content_repository_impl.dart';
import 'package:wallet/src/domain/model/help/help_topic_group.dart';
import 'package:wallet/src/domain/model/help/topic_block.dart';
import 'package:wallet/src/util/mapper/help/topic_block_mapper.dart';

/// In-memory [AssetBundle] — returns the string registered for an asset key
/// and throws [FlutterError] for anything else (mirroring what [rootBundle]
/// does at runtime for missing assets).
class _InMemoryAssetBundle extends CachingAssetBundle {
  final Map<String, String> _assets;

  _InMemoryAssetBundle(this._assets);

  @override
  Future<ByteData> load(String key) async {
    final value = _assets[key];
    if (value == null) throw FlutterError('Asset not found: $key');
    return ByteData.sublistView(Uint8List.fromList(utf8.encode(value)));
  }

  @override
  Future<String> loadString(String key, {bool cache = true}) async {
    final value = _assets[key];
    if (value == null) throw FlutterError('Asset not found: $key');
    return value;
  }
}

const _kYaml = '''
structure:
  - categoryId: getting_started
    icon: play_arrow
    subcategories:
      - subcategoryId: introduction
        topics:
          - groupId: help
            topicIds:
              - cannot_continue_demo
              - dont_know_wallet
          - groupId: information
            topicIds:
              - what_is_wallet
      - subcategoryId: digid
        topics:
          - groupId: help
            topicIds:
              - digid_does_not_open
translations:
  en:
    categories:
      getting_started:
        title: Getting started
        subtitle: Introduction and DigiD
    subcategories:
      introduction: Introduction
      digid: Start with DigiD
    topics:
      cannot_continue_demo: I cannot continue with the demo
      dont_know_wallet: I do not know what NL Wallet is
      what_is_wallet: What is NL Wallet?
      digid_does_not_open: DigiD does not open
  nl:
    categories:
      getting_started:
        title: Aan de slag
        subtitle: Introductie en DigiD
    subcategories:
      introduction: Introductie
      digid: Beginnen met DigiD
    topics:
      cannot_continue_demo: Ik kan niet verder met de demo
      dont_know_wallet: Ik weet niet wat NL Wallet is
      what_is_wallet: Wat is NL Wallet?
      digid_does_not_open: DigiD opent niet
''';

const _kTopicMarkdown = '''
NL Wallet is an app for your cards.

**How it works**

- Add a card.
- Share only what is needed.

[More info](help://dont_know_wallet)
''';

HelpContentRepositoryImpl _buildRepo(Map<String, String> assets) {
  return HelpContentRepositoryImpl(
    TopicBlockMapper(),
    bundle: _InMemoryAssetBundle(assets),
  );
}

void main() {
  group('getCategories', () {
    test('builds the full category tree with EN translations', () async {
      final repo = _buildRepo({'assets/non-free/markdown/help/help.yaml': _kYaml});

      final categories = await repo.getCategories(const Locale('en'));

      expect(categories, hasLength(1));
      final category = categories.single;
      expect(category.id, 'getting_started');
      expect(category.icon, 'play_arrow');
      expect(category.title, 'Getting started');
      expect(category.subtitle, 'Introduction and DigiD');
      expect(category.subcategories, hasLength(2));
    });

    test('applies NL translations when locale is NL', () async {
      final repo = _buildRepo({'assets/non-free/markdown/help/help.yaml': _kYaml});

      final categories = await repo.getCategories(const Locale('nl'));

      expect(categories.single.title, 'Aan de slag');
      expect(categories.single.subcategories[0].title, 'Introductie');
    });

    test('unknown locale falls back to EN translations', () async {
      final repo = _buildRepo({'assets/non-free/markdown/help/help.yaml': _kYaml});

      final categories = await repo.getCategories(const Locale('de'));

      // EN is the fallback — expect EN titles.
      expect(categories.single.title, 'Getting started');
    });

    test('parses topic groups with the right topics', () async {
      final repo = _buildRepo({'assets/non-free/markdown/help/help.yaml': _kYaml});

      final categories = await repo.getCategories(const Locale('en'));
      final introduction = categories.single.subcategories.firstWhere((s) => s.id == 'introduction');

      expect(introduction.groups.map((g) => g.kind), [HelpTopicGroupKind.help, HelpTopicGroupKind.information]);
      expect(
        introduction.groups.firstWhere((g) => g.kind == HelpTopicGroupKind.help).topics.map((t) => t.id),
        ['cannot_continue_demo', 'dont_know_wallet'],
      );
      expect(
        introduction.groups.firstWhere((g) => g.kind == HelpTopicGroupKind.information).topics.map((t) => t.id),
        ['what_is_wallet'],
      );
    });

    test('topic titles come from the selected locale', () async {
      final repo = _buildRepo({'assets/non-free/markdown/help/help.yaml': _kYaml});

      final en = await repo.getCategories(const Locale('en'));
      final nl = await repo.getCategories(const Locale('nl'));

      expect(en.single.subcategories[0].groups[0].topics[0].title, 'I cannot continue with the demo');
      expect(nl.single.subcategories[0].groups[0].topics[0].title, 'Ik kan niet verder met de demo');
    });

    test('subcategory that has only one group still parses that group', () async {
      final repo = _buildRepo({'assets/non-free/markdown/help/help.yaml': _kYaml});
      final categories = await repo.getCategories(const Locale('en'));

      final digid = categories.single.subcategories.firstWhere((s) => s.id == 'digid');
      expect(digid.groups, hasLength(1));
      expect(digid.groups.single.kind, HelpTopicGroupKind.help);
      expect(digid.groups.single.topics.map((t) => t.id), ['digid_does_not_open']);
    });

    test('caches the parsed tree per locale — second call does not reparse YAML', () async {
      var loadCount = 0;
      final bundle = _InMemoryAssetBundle({'assets/non-free/markdown/help/help.yaml': _kYaml});
      // Wrap the bundle to count YAML load calls.
      final countingBundle = _CountingAssetBundle(bundle, (key) {
        if (key.endsWith('help.yaml')) loadCount++;
      });
      final repo = HelpContentRepositoryImpl(TopicBlockMapper(), bundle: countingBundle);

      await repo.getCategories(const Locale('en'));
      await repo.getCategories(const Locale('en'));

      expect(loadCount, 1, reason: 'YAML should be loaded once, cached thereafter');
    });

    test('per-locale cache keeps EN and NL results independent', () async {
      final repo = _buildRepo({'assets/non-free/markdown/help/help.yaml': _kYaml});

      final enFirst = await repo.getCategories(const Locale('en'));
      final nlFirst = await repo.getCategories(const Locale('nl'));
      final enSecond = await repo.getCategories(const Locale('en'));

      expect(identical(enFirst, enSecond), isTrue, reason: 'EN call should return the cached list');
      expect(identical(enFirst, nlFirst), isFalse);
      expect(enFirst.single.title, 'Getting started');
      expect(nlFirst.single.title, 'Aan de slag');
    });
  });

  group('getTopicBlocks', () {
    test('loads the locale-specific markdown file and maps it to blocks', () async {
      final repo = _buildRepo({
        'assets/non-free/markdown/help/help.yaml': _kYaml,
        'assets/non-free/markdown/help/en/what_is_wallet.md': _kTopicMarkdown,
      });

      final blocks = await repo.getTopicBlocks('what_is_wallet', const Locale('en'));

      expect(blocks, hasLength(4));
      expect(blocks[0], isA<TopicParagraphBlock>());
      expect(blocks[1], isA<TopicHeadingBlock>());
      expect(blocks[2], isA<TopicBulletListBlock>());
      expect(blocks[3], isA<TopicReferenceBlock>());
      expect((blocks[3] as TopicReferenceBlock).links.single.topicId, 'dont_know_wallet');
    });

    test('uses the locale language code to select the markdown directory', () async {
      final repo = _buildRepo({
        'assets/non-free/markdown/help/help.yaml': _kYaml,
        'assets/non-free/markdown/help/nl/what_is_wallet.md': 'NL inhoud',
      });

      final blocks = await repo.getTopicBlocks('what_is_wallet', const Locale('nl'));

      expect(blocks.single, const TopicParagraphBlock('NL inhoud'));
    });

    test('propagates the bundle error when the markdown is missing', () async {
      final repo = _buildRepo({'assets/non-free/markdown/help/help.yaml': _kYaml});

      expect(
        () => repo.getTopicBlocks('what_is_wallet', const Locale('en')),
        throwsA(isA<FlutterError>()),
      );
    });
  });
}

/// Wraps a bundle and calls [_onLoad] for every `loadString` invocation.
/// Used to assert caching behaviour without reaching into the impl's internals.
class _CountingAssetBundle extends CachingAssetBundle {
  final AssetBundle _delegate;
  final void Function(String key) _onLoad;

  _CountingAssetBundle(this._delegate, this._onLoad);

  @override
  Future<ByteData> load(String key) => _delegate.load(key);

  @override
  Future<String> loadString(String key, {bool cache = true}) {
    _onLoad(key);
    return _delegate.loadString(key, cache: cache);
  }
}
