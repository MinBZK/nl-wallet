import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/widget/info_row.dart';

import '../../../../wallet_app_test_widget.dart';

void main() {
  const kGoldenSize = Size(300, 108);

  group('goldens', () {
    testGoldens(
      'light info row',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const InfoRow(
            title: Text('Title'),
            subtitle: Text('Subtitle'),
            icon: Icons.language_outlined,
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'info_row/light');
      },
    );
    testGoldens(
      'dark info row',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const InfoRow(
            title: Text('Title'),
            subtitle: Text('Subtitle'),
            icon: Icons.language_outlined,
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden(tester, 'info_row/dark');
      },
    );
    testGoldens(
      'light info row leading',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const InfoRow(
            title: Text('Title'),
            subtitle: Text('Subtitle'),
            leading: FlutterLogo(size: 50),
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'info_row/light.leading');
      },
    );

    testGoldens(
      'light info row no padding',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const InfoRow(
            title: Text('Title'),
            subtitle: Text('Subtitle'),
            icon: Icons.padding,
            padding: EdgeInsets.zero,
          ),
          surfaceSize: const Size(104, 80),
        );
        await screenMatchesGolden(tester, 'info_row/light.nopadding');
      },
    );
  });

  group('widgets', () {
    testWidgets('widgets are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const InfoRow(
          title: Text('T'),
          subtitle: Text('S'),
          icon: Icons.language_outlined,
        ),
      );

      // Validate that the widget exists
      final titleFinder = find.text('T');
      final subtitleFinder = find.text('S');
      expect(titleFinder, findsOneWidget);
      expect(subtitleFinder, findsOneWidget);
    });

    testWidgets('onTap is triggered', (tester) async {
      bool onTapCalled = false;
      await tester.pumpWidgetWithAppWrapper(
        InfoRow(
          title: const Text('T'),
          subtitle: const Text('S'),
          icon: Icons.language_outlined,
          onTap: () => onTapCalled = true,
        ),
      );

      // Validate that the widget exists
      final titleFinder = find.text('T');
      await tester.tap(titleFinder);

      expect(onTapCalled, isTrue);
    });
  });
}
