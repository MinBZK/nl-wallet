import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/widget/sliver_divider.dart';

import '../../../../wallet_app_test_widget.dart';

void main() {
  const kGoldenSize = Size(350, 10);

  group('goldens', () {
    testGoldens(
      'light sliver divider',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const CustomScrollView(
            slivers: [SliverDivider(height: 10)],
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'sliver_divider/light');
      },
    );
    testGoldens(
      'dark sliver divider',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const CustomScrollView(
            slivers: [SliverDivider(height: 10)],
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden(tester, 'sliver_divider/dark');
      },
    );
    testGoldens(
      'light sliver divider',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const CustomScrollView(
            slivers: [
              SliverDivider(
                height: 10,
                indent: 24,
                endIndent: 24,
              )
            ],
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'sliver_divider/light.indented');
      },
    );
  });
}
