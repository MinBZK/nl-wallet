import 'dart:ui';

import 'package:collection/collection.dart';
import 'package:flutter/services.dart';
import 'package:yaml/yaml.dart';

import '../../../../domain/model/help/help_category.dart';
import '../../../../domain/model/help/help_subcategory.dart';
import '../../../../domain/model/help/help_topic.dart';
import '../../../../domain/model/help/help_topic_group.dart';
import '../../../../domain/model/help/topic_block.dart';
import '../../../../util/mapper/mapper.dart';
import '../help_content_repository.dart';

class HelpContentRepositoryImpl implements HelpContentRepository {
  static const _helpRootDir = 'assets/non-free/markdown/help';
  static const _helpYamlAsset = '$_helpRootDir/help.yaml';

  final Mapper<String, List<TopicBlock>> _topicMarkdownMapper;
  final AssetBundle _bundle;

  YamlMap? _yamlCache;
  final Map<String, List<HelpCategory>> _categoriesCache = {};

  HelpContentRepositoryImpl(this._topicMarkdownMapper, {AssetBundle? bundle}) : _bundle = bundle ?? rootBundle;

  @override
  Future<List<HelpCategory>> getCategories(Locale locale) async {
    final code = locale.languageCode;
    final cached = _categoriesCache[code];
    if (cached != null) return cached;

    final yaml = await _loadYaml();
    final translations = _translationsFor(yaml, code);
    final structure = yaml['structure'] as YamlList;

    final categories = structure.map((entry) => _parseCategory(entry as YamlMap, translations)).toList();
    return _categoriesCache[code] = categories;
  }

  @override
  Future<List<TopicBlock>> getTopicBlocks(String topicId, Locale locale) async {
    final markdown = await _loadTopicMarkdown(topicId, locale.languageCode);
    return _topicMarkdownMapper.map(markdown);
  }

  Future<String> _loadTopicMarkdown(String topicId, String languageCode) {
    return _bundle.loadString('$_helpRootDir/$languageCode/$topicId.md');
  }

  Future<YamlMap> _loadYaml() async {
    if (_yamlCache != null) return _yamlCache!;
    final raw = await _bundle.loadString(_helpYamlAsset);
    _yamlCache = loadYaml(raw) as YamlMap;
    return _yamlCache!;
  }

  YamlMap _translationsFor(YamlMap yaml, String code) {
    final translations = yaml['translations'] as YamlMap;
    return (translations[code] ?? translations['en']) as YamlMap;
  }

  HelpCategory _parseCategory(YamlMap entry, YamlMap translations) {
    final id = entry['categoryId'] as String;
    final categoryTranslations = (translations['categories'] as YamlMap)[id] as YamlMap;
    final subcategories = (entry['subcategories'] as YamlList)
        .map((sub) => _parseSubcategory(sub as YamlMap, translations))
        .toList();

    return HelpCategory(
      id: id,
      icon: entry['icon'] as String,
      title: categoryTranslations['title'] as String,
      subtitle: categoryTranslations['subtitle'] as String,
      subcategories: subcategories,
    );
  }

  HelpSubcategory _parseSubcategory(YamlMap entry, YamlMap translations) {
    final id = entry['subcategoryId'] as String;
    final title = (translations['subcategories'] as YamlMap)[id] as String;
    final groups = _parseGroups(entry['topics'] as YamlList, translations['topics'] as YamlMap);

    return HelpSubcategory(id: id, title: title, groups: groups);
  }

  List<HelpTopicGroup> _parseGroups(YamlList entries, YamlMap topicTranslations) {
    return entries
        .map((entry) {
          final map = entry as YamlMap;
          final kind = _parseGroupKind(map['groupId'] as String);
          if (kind == null) return null;
          final topics = (map['topicIds'] as YamlList)
              .cast<String>()
              .map((topicId) => HelpTopic(id: topicId, title: topicTranslations[topicId] as String))
              .toList();
          return HelpTopicGroup(kind: kind, topics: topics);
        })
        .nonNulls
        .toList();
  }

  HelpTopicGroupKind? _parseGroupKind(String rawId) {
    return HelpTopicGroupKind.values.firstWhereOrNull((k) => k.name == rawId);
  }
}
