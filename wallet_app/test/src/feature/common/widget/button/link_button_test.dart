import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/widget/button/link_button.dart';

import '../../../../../wallet_app_test_widget.dart';

void main() {
  const kGoldenSize = Size(140, 50);

  group('goldens', () {
    testGoldens(
      'light button',
      (tester) async {
        await tester.pumpWidgetBuilder(
          LinkButton(
            onPressed: () {},
            child: const Text('Link Button'),
          ),
          wrapper: walletAppWrapper(brightness: Brightness.light),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'link_button/light');
      },
    );
    testGoldens(
      'dark button',
      (tester) async {
        await tester.pumpWidgetBuilder(
          LinkButton(
            onPressed: () {},
            child: const Text('Link Button'),
          ),
          wrapper: walletAppWrapper(brightness: Brightness.dark),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'link_button/dark');
      },
    );
    testGoldens(
      'light button zero padding',
      (tester) async {
        await tester.pumpWidgetBuilder(
          LinkButton(
            onPressed: () {},
            customPadding: EdgeInsets.zero,
            child: const Text('Link Button'),
          ),
          wrapper: walletAppWrapper(brightness: Brightness.light),
          surfaceSize: const Size(116, 30),
        );
        await screenMatchesGolden(tester, 'link_button/light.nopadding');
      },
    );
  });

  group('widgets', () {
    testWidgets('button is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        LinkButton(
          onPressed: () {},
          child: const Text('B'),
        ),
      );

      // Validate that the button exists
      final linkButtonFinder = find.text('B');
      expect(linkButtonFinder, findsOneWidget);
    });
  });
}
