/// Pure-Dart source of truth for the set of `icon` names that the help YAML
/// is allowed to use for a `categoryId`. Exposed as a plain [Set<String>] so
/// the validator (pure Dart, runs under `dart run`) can import it without
/// pulling in any Flutter package.
///
/// The corresponding name → [IconData] map lives in
/// `feature/help/help_category_icons.dart`. A drift test asserts that both
/// lists stay in sync.
const Set<String> allowedHelpCategoryIconNames = {
  'credit_card',
  'history',
  'lock_outline',
  'mobile_screen_share',
  'move_up',
  'qr_code',
  'settings',
  'start',
};
