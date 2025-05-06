import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/policy/widget/policy_entry_row.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';

const tourBannerSize = Size(240, 102);

void main() {
  group('goldens', () {
    testGoldens(
      'light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          PolicyEntryRow(
            title: Text('Title'),
            description: Text('Description'),
            icon: Icon(Icons.account_balance_wallet_rounded),
          ),
          surfaceSize: tourBannerSize,
        );
        await screenMatchesGolden('light');
      },
    );

    testGoldens(
      'light long text & description',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          PolicyEntryRow(
            title: Text('Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt.'),
            description: Text(
              'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt '
              'ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco '
              'laboris nisi ut aliquip ex ea commodo consequat.',
            ),
            icon: Icon(Icons.account_balance_wallet_rounded),
          ),
          surfaceSize: Size(200, 540),
        );
        await screenMatchesGolden('light.long');
      },
    );

    testGoldens(
      'light - no icon',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          PolicyEntryRow(
            title: Text('Title'),
            description: Text('Description'),
          ),
          surfaceSize: tourBannerSize,
        );
        await screenMatchesGolden('light.no_icon');
      },
    );

    testGoldens(
      'light scaled',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          PolicyEntryRow(
            title: Text('Title'),
            description: Text('Description'),
            icon: Icon(Icons.account_balance_wallet_rounded),
          ),
          surfaceSize: Size(300, 170),
          textScaleSize: 2.5,
        );
        await screenMatchesGolden('scaled.light');
      },
    );

    testGoldens(
      'dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          PolicyEntryRow(
            title: Text('Title'),
            description: Text('Description'),
            icon: Icon(Icons.account_balance_wallet_rounded),
          ),
          brightness: Brightness.dark,
          surfaceSize: tourBannerSize,
        );
        await screenMatchesGolden('dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('banner shows title and description', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PolicyEntryRow(
          title: Text('Title'),
          description: Text('Description'),
          icon: Icon(Icons.account_balance_wallet_rounded),
        ),
      );

      final titleFinder = find.text('Title');
      final descriptionFinder = find.text('Description');
      expect(titleFinder, findsOneWidget);
      expect(descriptionFinder, findsOneWidget);
    });
  });
}
