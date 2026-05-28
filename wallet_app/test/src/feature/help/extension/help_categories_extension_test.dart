import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/help/help_category.dart';
import 'package:wallet/src/feature/help/extension/help_categories_extension.dart';

import '../../../mocks/wallet_mock_data.dart';

void main() {
  group('HelpCategoriesFindTopic.findTopicTitle', () {
    test('returns the topic title for a known topicId', () {
      expect(WalletMockData.helpCategories.findTopicTitle('what_is_wallet'), 'What is NL Wallet?');
    });

    test('returns null for an unknown topicId', () {
      expect(WalletMockData.helpCategories.findTopicTitle('does_not_exist'), isNull);
    });

    test('returns null when called on an empty taxonomy', () {
      expect(const <HelpCategory>[].findTopicTitle('anything'), isNull);
    });
  });

  group('helpCategoryIcons', () {
    test('every value is a non-null IconData', () {
      for (final entry in helpCategoryIcons.entries) {
        expect(entry.value, isA<IconData>(), reason: 'icon for "${entry.key}"');
      }
    });
  });

  group('HelpCategoryIcon.iconData', () {
    HelpCategory categoryWithIcon(String icon) => HelpCategory(
      id: 'cat',
      icon: icon,
      title: 'title',
      subtitle: 'subtitle',
      subcategories: const [],
    );

    test('returns the mapped icon for every supported name', () {
      for (final name in helpCategoryIcons.keys) {
        expect(categoryWithIcon(name).iconData, helpCategoryIcons[name]);
      }
    });

    test('falls back to Icons.help_outline for unknown names', () {
      expect(categoryWithIcon('definitely_not_in_the_map').iconData, Icons.help_outline);
    });
  });
}
