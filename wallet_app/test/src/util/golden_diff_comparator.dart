import 'dart:developer';

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';

class GoldenDiffComparator extends LocalFileComparator {
  final String testDir;
  final double tolerance;

  GoldenDiffComparator(
    this.testDir, {
    this.tolerance = 0.0025 /* 0.25% */,
  }) : super(Uri.parse(testDir));

  @override
  Future<void> update(Uri golden, Uint8List imageBytes) =>
      super.update(Uri.parse(testDir + golden.toString()), imageBytes);

  @override
  Future<bool> compare(Uint8List imageBytes, Uri fileName) async {
    final goldenPath = Uri.parse(testDir + fileName.toString());
    final ComparisonResult result = await GoldenFileComparator.compareLists(
      imageBytes,
      await getGoldenBytes(goldenPath),
    );

    // Everything looks great!
    if (result.passed) return true;

    // Test didn't pass, check if it's within our tolerance levels
    if (result.diffPercent <= tolerance) {
      log('A tolerable difference of ${result.diffPercent * 100}% was found when comparing $goldenPath.');
      return true;
    }

    // Test didn't pass and doesn't fall within the provided tolerance level, fail test.
    final String error = await generateFailureOutput(result, fileName, Uri.parse(testDir));
    throw FlutterError(error);
  }
}
