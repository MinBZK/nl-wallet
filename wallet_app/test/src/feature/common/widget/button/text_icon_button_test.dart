import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/widget/button/text_icon_button.dart';

import '../../../../../wallet_app_test_widget.dart';

void main() {
  const kGoldenSize = Size(136, 50);

  group('goldens', () {
    testGoldens(
      'light button',
      (tester) async {
        await tester.pumpWidgetBuilder(
          TextIconButton(
            onPressed: () {},
            child: const Text('Link'),
          ),
          wrapper: walletAppWrapper(brightness: Brightness.light),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'text_icon_button/light');
      },
    );
    testGoldens(
      'dark button',
      (tester) async {
        await tester.pumpWidgetBuilder(
          TextIconButton(
            onPressed: () {},
            child: const Text('Link'),
          ),
          wrapper: walletAppWrapper(brightness: Brightness.dark),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'text_icon_button/dark');
      },
    );

    testGoldens(
      'light button not centered',
      (tester) async {
        await tester.pumpWidgetBuilder(
          TextIconButton(
            onPressed: () {},
            centerChild: false,
            child: const Text('Link'),
          ),
          wrapper: walletAppWrapper(brightness: Brightness.light),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'text_icon_button/light.offcenter');
      },
    );

    testGoldens(
      'light button icon on the left',
      (tester) async {
        await tester.pumpWidgetBuilder(
          TextIconButton(
            onPressed: () {},
            iconPosition: IconPosition.start,
            child: const Text('Link'),
          ),
          wrapper: walletAppWrapper(brightness: Brightness.light),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'text_icon_button/light.left');
      },
    );

    testGoldens(
      'light button custom icon',
      (tester) async {
        await tester.pumpWidgetBuilder(
          TextIconButton(
            onPressed: () {},
            icon: Icons.language_outlined,
            child: const Text('Link'),
          ),
          wrapper: walletAppWrapper(brightness: Brightness.light),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'text_icon_button/light.icon');
      },
    );
  });

  group('widgets', () {
    testWidgets('button is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        TextIconButton(
          onPressed: () {},
          child: const Text('B'),
        ),
      );

      // Validate that the button exists
      final buttonFinder = find.text('B');
      expect(buttonFinder, findsOneWidget);
    });
  });
}
