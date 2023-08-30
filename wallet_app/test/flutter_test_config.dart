import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';

import 'src/util/golden_diff_comparator.dart';

Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  await loadAppFonts();
  _setupGoldenFileComparator();
  return testMain();
}

/// Overrides the default [LocalFileComparator] with our [GoldenDiffComparator] that has
/// a configurable tolerance (defaults to 0.5%) when comparing goldens.
void _setupGoldenFileComparator() {
  final testFilePath = (goldenFileComparator as LocalFileComparator).basedir;
  goldenFileComparator = GoldenDiffComparator(testFilePath.toString());
}
