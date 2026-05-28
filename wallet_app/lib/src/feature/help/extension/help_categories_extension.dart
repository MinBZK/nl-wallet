import 'package:flutter/material.dart';

import '../../../domain/model/help/help_category.dart';

/// Maps YAML `icon` names to the [IconData] rendered for a [HelpCategory].
/// Keep the keys in sync with `allowedHelpCategoryIconNames` — the validator
/// (and the `verify-help-content` CI job) relies on that set and a drift test
/// asserts that both lists stay in sync.
const Map<String, IconData> helpCategoryIcons = {
  'credit_card': Icons.credit_card_outlined,
  'history': Icons.history,
  'lock_outline': Icons.lock_outline,
  'mobile_screen_share': Icons.mobile_screen_share_outlined,
  'move_up': Icons.move_up_outlined,
  'qr_code': Icons.qr_code,
  'settings': Icons.settings_outlined,
  'start': Icons.start_outlined,
};

extension HelpCategoryIcon on HelpCategory {
  /// Resolves the YAML icon name to an [IconData]. Falls back to
  /// [Icons.help_outline] for unknown names; the validator rejects unknown
  /// names at content-edit time, so the fallback should be unreachable in
  /// production — it's a last-resort safety net.
  IconData get iconData => helpCategoryIcons[icon] ?? Icons.help_outline;
}

extension HelpCategoriesFindTopic on Iterable<HelpCategory> {
  /// Walks the help taxonomy looking for a topic with the given [topicId] and
  /// returns its localized title, or `null` if no match is found.
  String? findTopicTitle(String topicId) {
    for (final category in this) {
      for (final subcategory in category.subcategories) {
        for (final group in subcategory.groups) {
          for (final topic in group.topics) {
            if (topic.id == topicId) return topic.title;
          }
        }
      }
    }
    return null;
  }
}
