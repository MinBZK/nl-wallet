import 'package:flutter_test/flutter_test.dart' as test_package;
import 'package:flutter_test/flutter_test.dart';
import 'package:meta/meta.dart';

import '../test_extension/common_finders_extension.dart';

@isTest
void testGoldens(
  String description,
  WidgetTesterCallback callback, {
  bool? skip,
  test_package.Timeout? timeout,
  bool semanticsEnabled = true,
  TestVariant<Object?> variant = const DefaultTestVariant(),
  tags = const ['golden'],
  int? retry,
}) {
  testWidgets(
    description,
    callback,
    skip: skip,
    timeout: timeout,
    semanticsEnabled: semanticsEnabled,
    variant: variant,
    tags: tags,
  );
}

Future<void> screenMatchesGolden(String name) async => expectLater(find.root, matchesGoldenFile('goldens/$name.png'));
