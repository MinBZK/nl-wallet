import 'dart:io';

import 'package:yaml/yaml.dart';

import '../domain/model/help/help_topic_group.dart';
import '../domain/model/help/topic_block.dart';
import 'help_category_icon_names.dart';
import 'mapper/help/topic_block_mapper.dart';

/// Locales the help corpus must provide content for. Keep in sync with
/// `pubspec.yaml` asset declarations and the per-locale markdown directories.
const _kSupportedLocales = ['en', 'nl'];

/// File extension for help topic content files.
const _kMarkdownExtension = '.md';

/// Filename of the help content manifest. Used as the `file` field on issues
/// that originate from the YAML manifest rather than a markdown file.
const _kHelpYamlFile = 'help.yaml';

class ContentIssue {
  final String file;
  final String message;

  const ContentIssue({required this.file, required this.message});
}

class ValidationResult {
  final List<ContentIssue> issues;

  const ValidationResult(this.issues);

  bool get hasIssues => issues.isNotEmpty;
}

/// Validates the help content corpus under [helpDir] (the directory that
/// contains `help.yaml` plus per-locale markdown subdirectories).
///
/// Issues (block CI): missing markdown, orphan markdown, broken `help://`
/// links, missing/blank translations, unknown category `icon` names, unknown
/// topic-group ids.
ValidationResult validateHelpContent({required String helpDir}) {
  final issues = <ContentIssue>[];
  final yaml = loadYaml(File('$helpDir/$_kHelpYamlFile').readAsStringSync()) as YamlMap;

  final topicIds = _collectTopicIds(yaml);
  final categoryIds = _collectCategoryIds(yaml);
  final subcategoryIds = _collectSubcategoryIds(yaml);

  _checkMarkdownFilesPresent(helpDir, topicIds, issues);
  _checkNoOrphanMarkdown(helpDir, topicIds, issues);
  _checkHelpLinksResolve(helpDir, topicIds, issues);
  _checkTranslations(yaml, categoryIds, subcategoryIds, topicIds, issues);
  _checkCategoryIcons(yaml, issues);
  _checkGroupIds(yaml, issues);

  return ValidationResult(issues);
}

Set<String> _collectTopicIds(YamlMap yaml) {
  final structure = yaml['structure'] as YamlList;
  return structure
      .cast<YamlMap>()
      .expand((category) => (category['subcategories'] as YamlList).cast<YamlMap>())
      .expand((sub) => (sub['topics'] as YamlList).cast<YamlMap>())
      .expand((group) => (group['topicIds'] as YamlList).whereType<String>())
      .toSet();
}

Set<String> _collectCategoryIds(YamlMap yaml) {
  final structure = yaml['structure'] as YamlList;
  return structure.cast<YamlMap>().map((category) => category['categoryId'] as String).toSet();
}

Set<String> _collectSubcategoryIds(YamlMap yaml) {
  final structure = yaml['structure'] as YamlList;
  return structure
      .cast<YamlMap>()
      .expand((category) => (category['subcategories'] as YamlList).cast<YamlMap>())
      .map((sub) => sub['subcategoryId'] as String)
      .toSet();
}

void _checkMarkdownFilesPresent(String helpDir, Set<String> topicIds, List<ContentIssue> issues) {
  for (final id in topicIds) {
    for (final locale in _kSupportedLocales) {
      if (!File('$helpDir/$locale/$id$_kMarkdownExtension').existsSync()) {
        issues.add(
          ContentIssue(
            file: _kHelpYamlFile,
            message: "topicId '$id' has no $locale/$id$_kMarkdownExtension",
          ),
        );
      }
    }
  }
}

void _checkNoOrphanMarkdown(String helpDir, Set<String> topicIds, List<ContentIssue> issues) {
  for (final locale in _kSupportedLocales) {
    final dir = Directory('$helpDir/$locale');
    if (!dir.existsSync()) continue;
    for (final entity in dir.listSync()) {
      if (entity is! File || !entity.path.endsWith(_kMarkdownExtension)) continue;
      final name = entity.uri.pathSegments.last;
      final id = name.substring(0, name.length - _kMarkdownExtension.length);
      if (!topicIds.contains(id)) {
        issues.add(
          ContentIssue(
            file: '$locale/$name',
            message: "orphan markdown — no topicId '$id' in $_kHelpYamlFile",
          ),
        );
      }
    }
  }
}

/// Stateless parser reused across every topic file; only static regexes and
/// pure methods, so a single shared instance is safe.
final _topicBlockMapper = TopicBlockMapper();

void _checkHelpLinksResolve(String helpDir, Set<String> topicIds, List<ContentIssue> issues) {
  for (final locale in _kSupportedLocales) {
    for (final id in topicIds) {
      _checkLinksInTopicFile(helpDir, locale, id, topicIds, issues);
    }
  }
}

void _checkLinksInTopicFile(
  String helpDir,
  String locale,
  String id,
  Set<String> topicIds,
  List<ContentIssue> issues,
) {
  final file = File('$helpDir/$locale/$id$_kMarkdownExtension');
  if (!file.existsSync()) return;
  for (final block in _topicBlockMapper.map(file.readAsStringSync())) {
    if (block is! TopicReferenceBlock) continue;
    for (final link in block.links) {
      if (topicIds.contains(link.topicId)) continue;
      issues.add(
        ContentIssue(
          file: '$locale/$id$_kMarkdownExtension',
          message: 'help://${link.topicId} — unknown topicId',
        ),
      );
    }
  }
}

bool _isMissingOrBlank(Object? value) => value is! String || value.trim().isEmpty;

void _checkTranslations(
  YamlMap yaml,
  Set<String> categoryIds,
  Set<String> subcategoryIds,
  Set<String> topicIds,
  List<ContentIssue> issues,
) {
  final translations = yaml['translations'] as YamlMap?;
  if (translations == null) {
    issues.add(
      const ContentIssue(
        file: _kHelpYamlFile,
        message: 'missing top-level translations block',
      ),
    );
    return;
  }

  for (final locale in _kSupportedLocales) {
    final localeMap = translations[locale] as YamlMap?;
    if (localeMap == null) {
      issues.add(
        ContentIssue(
          file: _kHelpYamlFile,
          message: 'missing translations.$locale block',
        ),
      );
      continue;
    }
    _checkCategoryTranslations(localeMap, locale, categoryIds, issues);
    _checkSubcategoryTranslations(localeMap, locale, subcategoryIds, issues);
    _checkTopicTranslations(localeMap, locale, topicIds, issues);
  }
}

void _checkCategoryTranslations(
  YamlMap localeMap,
  String locale,
  Set<String> categoryIds,
  List<ContentIssue> issues,
) {
  final cats = (localeMap['categories'] as YamlMap?) ?? YamlMap();
  for (final id in categoryIds) {
    final entry = cats[id];
    if (entry is! YamlMap) {
      issues.add(
        ContentIssue(
          file: _kHelpYamlFile,
          message: "category '$id' has no $locale translation",
        ),
      );
      continue;
    }
    if (_isMissingOrBlank(entry['title'])) {
      issues.add(
        ContentIssue(
          file: _kHelpYamlFile,
          message: "category '$id' has missing/blank $locale.title",
        ),
      );
    }
    if (_isMissingOrBlank(entry['subtitle'])) {
      issues.add(
        ContentIssue(
          file: _kHelpYamlFile,
          message: "category '$id' has missing/blank $locale.subtitle",
        ),
      );
    }
  }
}

void _checkSubcategoryTranslations(
  YamlMap localeMap,
  String locale,
  Set<String> subcategoryIds,
  List<ContentIssue> issues,
) {
  final subs = (localeMap['subcategories'] as YamlMap?) ?? YamlMap();
  for (final id in subcategoryIds) {
    if (_isMissingOrBlank(subs[id])) {
      issues.add(
        ContentIssue(
          file: _kHelpYamlFile,
          message: "subcategory '$id' has missing/blank $locale translation",
        ),
      );
    }
  }
}

void _checkTopicTranslations(
  YamlMap localeMap,
  String locale,
  Set<String> topicIds,
  List<ContentIssue> issues,
) {
  final topics = (localeMap['topics'] as YamlMap?) ?? YamlMap();
  for (final id in topicIds) {
    if (_isMissingOrBlank(topics[id])) {
      issues.add(
        ContentIssue(
          file: _kHelpYamlFile,
          message: "topic '$id' has missing/blank $locale translation",
        ),
      );
    }
  }
}

void _checkCategoryIcons(YamlMap yaml, List<ContentIssue> issues) {
  for (final category in yaml['structure'] as YamlList) {
    final entry = category as YamlMap;
    final categoryId = entry['categoryId'] as String;
    final icon = entry['icon'];
    if (icon is! String) {
      issues.add(
        ContentIssue(
          file: _kHelpYamlFile,
          message: "category '$categoryId' has no icon",
        ),
      );
      continue;
    }
    if (!allowedHelpCategoryIconNames.contains(icon)) {
      issues.add(
        ContentIssue(
          file: _kHelpYamlFile,
          message:
              "category '$categoryId' uses unknown icon '$icon' — add it to `helpCategoryIcons` and `allowedHelpCategoryIconNames` (Dart change required).",
        ),
      );
    }
  }
}

/// Topic-group ids the app understands, derived from [HelpTopicGroupKind].
/// The repository parser silently drops a group whose `groupId` is anything
/// else, so an unknown id means its topics never reach a screen.
final _kValidGroupIds = HelpTopicGroupKind.values.map((kind) => kind.name).toSet();

void _checkGroupIds(YamlMap yaml, List<ContentIssue> issues) {
  for (final category in yaml['structure'] as YamlList) {
    for (final sub in (category as YamlMap)['subcategories'] as YamlList) {
      final subMap = sub as YamlMap;
      final subcategoryId = subMap['subcategoryId'];
      for (final group in subMap['topics'] as YamlList) {
        final groupId = (group as YamlMap)['groupId'];
        if (groupId is String && _kValidGroupIds.contains(groupId)) continue;
        issues.add(
          ContentIssue(
            file: _kHelpYamlFile,
            message:
                "subcategory '$subcategoryId' uses unknown groupId '$groupId' — expected one of: ${_kValidGroupIds.join(', ')}",
          ),
        );
      }
    }
  }
}
