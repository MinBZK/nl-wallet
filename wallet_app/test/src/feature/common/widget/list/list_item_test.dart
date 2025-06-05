import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/button/link_button.dart';
import 'package:wallet/src/feature/common/widget/button/list_button.dart';
import 'package:wallet/src/feature/common/widget/list/compact_list_item.dart';
import 'package:wallet/src/feature/common/widget/list/horizontal_list_item.dart';
import 'package:wallet/src/feature/common/widget/list/list_item.dart';
import 'package:wallet/src/feature/common/widget/list/vertical_list_item.dart';
import 'package:wallet/src/wallet_assets.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../test_util/golden_utils.dart';

void main() {
  const kGoldenSize = Size(320, 42);

  group('goldens', () {
    testGoldens(
      'Icon, Label, Subtitle - Compact - Light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ListItem(
            icon: Icon(Icons.history),
            label: Text('Label'),
            subtitle: Text('Subtitle'),
            style: ListItemStyle.compact,
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.light,
        );
        await screenMatchesGolden('list_item/icon.label.subtitle.compact.light');
      },
    );

    testGoldens(
      'Icon, Label, Subtitle - Horizontal - Light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ListItem(
            icon: Icon(Icons.history),
            label: Text('Label'),
            subtitle: Text('Subtitle'),
            style: ListItemStyle.horizontal,
          ),
          surfaceSize: Size(kGoldenSize.width, 100),
          brightness: Brightness.light,
        );
        await screenMatchesGolden('list_item/icon.label.subtitle.horizontal.light');
      },
    );

    testGoldens(
      'Icon, Label, Subtitle - Vertical - Light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ListItem(
            icon: Icon(Icons.history),
            label: Text('Label'),
            subtitle: Text('Subtitle'),
            style: ListItemStyle.vertical,
          ),
          surfaceSize: Size(kGoldenSize.width, 142),
          brightness: Brightness.light,
        );
        await screenMatchesGolden('list_item/icon.label.subtitle.vertical.light');
      },
    );

    testGoldens(
      'Icon, Label, Subtitle - Compact - Light - Scaled x3',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ListItem(
            icon: Icon(Icons.history),
            label: Text('Label'),
            subtitle: Text('Subtitle'),
            style: ListItemStyle.compact,
          ),
          surfaceSize: Size(kGoldenSize.width, 142),
          brightness: Brightness.light,
          textScaleSize: 3,
        );
        await screenMatchesGolden('list_item/icon.label.subtitle.compact.light.scaled');
      },
    );

    testGoldens(
      'Image, Long Label, Long Subtitle - Horizontal - Light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          ListItem(
            icon: Image.asset(WalletAssets.logo_ecommerce),
            label: const Text('Lorem ipsum dolor sit amet, consectetur adipiscing elit'),
            subtitle: const Text(
              'Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.',
            ),
            style: ListItemStyle.horizontal,
          ),
          surfaceSize: Size(kGoldenSize.width, 166),
          brightness: Brightness.light,
        );
        await screenMatchesGolden('list_item/image.long_label.long_subtitle.horizontal.light');
      },
    );

    testGoldens(
      'Icon, Label, Subtitle - Compact - Dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ListItem(
            icon: Icon(Icons.history),
            label: Text('Label'),
            subtitle: Text('Subtitle'),
            style: ListItemStyle.compact,
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('list_item/icon.label.subtitle.compact.dark');
      },
    );

    testGoldens(
      'Icon, Label, Subtitle - Horizontal - Dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ListItem(
            icon: Icon(Icons.details_outlined),
            label: Text('Label'),
            subtitle: Text('Subtitle'),
            style: ListItemStyle.horizontal,
          ),
          surfaceSize: Size(kGoldenSize.width, 100),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('list_item/icon.label.subtitle.horizontal.dark');
      },
    );

    testGoldens(
      'Icon, Label, Subtitle - Vertical - Dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ListItem(
            icon: Icon(Icons.settings),
            label: Text('Label'),
            subtitle: Text('Subtitle'),
            style: ListItemStyle.vertical,
          ),
          surfaceSize: Size(kGoldenSize.width, 142),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('list_item/icon.label.subtitle.vertical.dark');
      },
    );

    testGoldens(
      'Icon, Label, Subtitle - Compact - Top Divider - Light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ListItem(
            icon: Icon(Icons.history),
            label: Text('Label'),
            subtitle: Text('Subtitle'),
            style: ListItemStyle.compact,
            dividerSide: DividerSide.top,
          ),
          surfaceSize: Size(kGoldenSize.width, 43),
          brightness: Brightness.light,
        );
        await screenMatchesGolden('list_item/icon.label.subtitle.compact.top_divider.light');
      },
    );

    testGoldens(
      'Icon, Label, Subtitle - Compact - Bottom Divider - Light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ListItem(
            icon: Icon(Icons.history),
            label: Text('Label'),
            subtitle: Text('Subtitle'),
            style: ListItemStyle.compact,
            dividerSide: DividerSide.bottom,
          ),
          surfaceSize: Size(kGoldenSize.width, 43),
          brightness: Brightness.light,
        );
        await screenMatchesGolden('list_item/icon.label.subtitle.compact.bottom_divider.light');
      },
    );

    testGoldens(
      'Icon, Label, Subtitle - Compact - Both Dividers - Light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const ListItem(
            icon: Icon(Icons.history),
            label: Text('Label'),
            subtitle: Text('Subtitle'),
            style: ListItemStyle.compact,
            dividerSide: DividerSide.both,
          ),
          surfaceSize: Size(kGoldenSize.width, 44),
          brightness: Brightness.light,
        );
        await screenMatchesGolden('list_item/icon.label.subtitle.compact.both_dividers.light');
      },
    );

    testGoldens(
      'Multi item (different styles)',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              ListItem(
                icon: Container(color: Colors.blueAccent),
                label: const Text('Compact Style'),
                subtitle: const Text('This is a compact list item'),
                style: ListItemStyle.compact,
                dividerSide: DividerSide.bottom,
              ),
              ListItem(
                icon: Container(color: Colors.amber),
                label: const Text('Horizontal Style'),
                subtitle: const Text('This is a horizontal list item with more padding'),
                style: ListItemStyle.horizontal,
                dividerSide: DividerSide.bottom,
              ),
              ListItem(
                icon: Container(color: Colors.green),
                label: const Text('Vertical Style'),
                subtitle: const Text('This is a vertical list item with different layout'),
                style: ListItemStyle.vertical,
              ),
            ],
          ),
          surfaceSize: Size(kGoldenSize.width, 330),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('list_item/multi_item_different_styles.dark');
      },
    );
  });

  testGoldens(
    'No Icon, Label, Subtitle - Compact - Light',
    (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ListItem(
          label: Text('Label'),
          subtitle: Text('Subtitle'),
          style: ListItemStyle.compact,
          dividerSide: DividerSide.both,
        ),
        surfaceSize: Size(kGoldenSize.width, 44),
        brightness: Brightness.light,
      );
      await screenMatchesGolden('list_item/no_icon.label.subtitle.compact.light');
    },
  );

  testGoldens(
    'No Icon, Label, Subtitle - Horizontal - Light',
    (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ListItem(
          label: Text('Label'),
          subtitle: Text('Subtitle'),
          style: ListItemStyle.horizontal,
          dividerSide: DividerSide.both,
        ),
        surfaceSize: Size(kGoldenSize.width, 102),
        brightness: Brightness.light,
      );
      await screenMatchesGolden('list_item/no_icon.label.subtitle.horizontal.light');
    },
  );

  testGoldens(
    'No Icon, Label, Subtitle - Vertical - Light',
    (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ListItem(
          label: Text('Label'),
          subtitle: Text('Subtitle'),
          style: ListItemStyle.vertical,
          dividerSide: DividerSide.both,
        ),
        surfaceSize: Size(kGoldenSize.width, 104),
        brightness: Brightness.light,
      );
      await screenMatchesGolden('list_item/no_icon.label.subtitle.vertical.light');
    },
  );

  testGoldens(
    'Icon, Label, Subtitle & Button - Vertical - Light',
    (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        ListItem(
          icon: const Icon(Icons.history),
          label: const Text('Label'),
          subtitle: const Text('Subtitle'),
          button: LinkButton(
            text: const Text('Link Button'),
            onPressed: () {},
          ),
          style: ListItemStyle.vertical,
        ),
        surfaceSize: Size(kGoldenSize.width, 202),
        brightness: Brightness.light,
      );
      await screenMatchesGolden('list_item/icon.label.subtitle.button.vertical.light');
    },
  );

  testGoldens(
    'Icon, Label, Subtitle & Button - Vertical - Light & Focused',
    (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        ListItem(
          icon: const Icon(Icons.history),
          label: const Text('Label'),
          subtitle: const Text('Subtitle'),
          button: LinkButton(
            text: const Text('Link Button'),
            onPressed: () {},
          ),
          style: ListItemStyle.vertical,
        ),
        surfaceSize: Size(kGoldenSize.width, 202),
        brightness: Brightness.light,
      );
      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await tester.pumpAndSettle();
      await screenMatchesGolden('list_item/icon.label.subtitle.button.vertical.light.focused');
    },
  );

  testWidgets('Verify compact constructor uses CompactListItem', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      const ListItem.compact(
        label: Text('Label'),
        subtitle: Text('Subtitle'),
      ),
      surfaceSize: kGoldenSize,
    );
    expect(find.byType(CompactListItem), findsOneWidget);
    expect(find.byType(HorizontalListItem), findsNothing);
    expect(find.byType(VerticalListItem), findsNothing);
  });

  testWidgets('Verify horizontal constructor uses HorizontalListItem', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      const ListItem.horizontal(
        label: Text('Label'),
        subtitle: Text('Subtitle'),
      ),
      surfaceSize: Size(kGoldenSize.width, 100),
    );
    expect(find.byType(HorizontalListItem), findsOneWidget);
    expect(find.byType(CompactListItem), findsNothing);
    expect(find.byType(VerticalListItem), findsNothing);
  });

  testWidgets('Verify vertical constructor uses VerticalListItem', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      const ListItem.vertical(
        label: Text('Label'),
        subtitle: Text('Subtitle'),
      ),
      surfaceSize: Size(kGoldenSize.width, 142),
    );
    expect(find.byType(VerticalListItem), findsOneWidget);
    expect(find.byType(HorizontalListItem), findsNothing);
    expect(find.byType(CompactListItem), findsNothing);
  });
}
