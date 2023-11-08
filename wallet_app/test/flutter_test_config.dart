import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/attribute/data_attribute.dart';
import 'package:wallet/src/domain/model/card_front.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/util/extension/string_extension.dart';

import 'src/util/golden_diff_comparator.dart';

Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  await loadAppFonts();
  _setupMockitoDummies();
  _setupGoldenFileComparator();
  return testMain();
}

// Configure some basic mockito dummies
void _setupMockitoDummies() {
  provideDummy<DataAttribute>(
    DataAttribute.untranslated(key: '', label: '', value: const StringValue(''), sourceCardId: ''),
  );
  provideDummy<AttributeValue>(const StringValue(''));
  provideDummy<CardFront>(CardFront(title: ''.untranslated, backgroundImage: '', theme: CardFrontTheme.light));
  provideDummy<NavigationRequest>(const GenericNavigationRequest('/mock_destination'));
}

/// Overrides the default [LocalFileComparator] with our [GoldenDiffComparator] that has
/// a configurable tolerance (defaults to 0.5%) when comparing goldens.
void _setupGoldenFileComparator() {
  final testFilePath = (goldenFileComparator as LocalFileComparator).basedir;
  goldenFileComparator = GoldenDiffComparator(testFilePath.toString());
}
