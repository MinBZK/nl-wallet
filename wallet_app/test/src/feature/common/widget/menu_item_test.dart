import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/button/list_button.dart';
import 'package:wallet/src/feature/common/widget/menu_item.dart';
import 'package:wallet/src/feature/common/widget/organization/organization_logo.dart';
import 'package:wallet/src/wallet_assets.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  const kGoldenSize = Size(320, 80);

  group('goldens', () {
    testGoldens(
      'LeftIcon, Label, Subtitle - Light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          MenuItem(
            leftIcon: const Icon(Icons.history),
            label: const Text('Label'),
            subtitle: const Text('Subtitle'),
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.light,
        );
        await screenMatchesGolden('menu_item/icon.label.subtitle.light');
      },
    );

    testGoldens(
      'LeftIcon, Label, Subtitle - Light - Scaled x3',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          MenuItem(
            leftIcon: const Icon(Icons.history),
            label: const Text('Label'),
            subtitle: const Text('Subtitle'),
            onPressed: () {},
          ),
          surfaceSize: Size(kGoldenSize.width, 158),
          brightness: Brightness.light,
          textScaleSize: 3,
        );
        await screenMatchesGolden('menu_item/icon.label.subtitle.light.scaled');
      },
    );

    testGoldens(
      'LeftIcon, Label, Subtitle, Underline - Light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          MenuItem(
            leftIcon: const Icon(Icons.history),
            label: const Text('Label'),
            subtitle: const Text('Subtitle'),
            underline: const Text('Underline'),
            onPressed: () {},
          ),
          surfaceSize: Size(kGoldenSize.width, 90),
          brightness: Brightness.light,
        );
        await screenMatchesGolden('menu_item/icon.label.subtitle.underline.light');
      },
    );

    testGoldens(
      'LeftIcon, Label, Subtitle, Underline, Error - Light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          MenuItem(
            leftIcon: const Icon(Icons.history),
            label: const Text('Label'),
            subtitle: const Text('Subtitle'),
            underline: const Text('Underline'),
            errorIcon: const Icon(Icons.block_flipped),
            onPressed: () {},
          ),
          surfaceSize: Size(kGoldenSize.width, 90),
          brightness: Brightness.light,
        );
        await screenMatchesGolden('menu_item/icon.label.subtitle.underline.error.light');
      },
    );

    testGoldens(
      'Label, Subtitle, Underline - Light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          MenuItem(
            label: const Text('Label'),
            subtitle: const Text('Subtitle'),
            underline: const Text('Underline'),
            onPressed: () {},
          ),
          surfaceSize: Size(kGoldenSize.width, 90),
          brightness: Brightness.light,
        );
        await screenMatchesGolden('menu_item/label.subtitle.underline.light');
      },
    );

    testGoldens(
      'Label - Light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          MenuItem(
            label: const Text('Label'),
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.light,
        );
        await screenMatchesGolden('menu_item/label.light');
      },
    );

    testGoldens(
      'Subtitle - Light',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          MenuItem(
            subtitle: const Text('Subtitle'),
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.light,
        );
        await screenMatchesGolden('menu_item/subtitle.light');
      },
    );

    testGoldens(
      'Large, LeftIcon, Label, Subtitle - Light',
      (tester) async {
        /// Note that the icon itself is not scaled, only the container. This should mainly be used for logos.
        await tester.pumpWidgetWithAppWrapper(
          MenuItem(
            leftIcon: const Icon(Icons.history),
            label: const Text('Label'),
            subtitle: const Text('Subtitle'),
            largeIcon: true,
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.light,
        );
        await screenMatchesGolden('menu_item/large.icon.label.subtitle.light');
      },
    );

    testGoldens(
      'LeftLogo, Label, Subtitle - Light',
      (tester) async {
        /// Note that the icon itself is not scaled, only the container. This should mainly be used for logos.
        await tester.pumpWidgetWithAppWrapper(
          MenuItem(
            leftIcon: OrganizationLogo(image: WalletMockData.organization.logo, size: kMenuItemNormalIconSize),
            label: const Text('Label'),
            subtitle: const Text('Subtitle'),
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.light,
        );
        await screenMatchesGolden('menu_item/logo.label.subtitle.light');
      },
    );

    testGoldens(
      'Large, LeftLogo, Label, Subtitle - Light',
      (tester) async {
        /// Note that the icon itself is not scaled, only the container. This should mainly be used for logos.
        await tester.pumpWidgetWithAppWrapper(
          MenuItem(
            leftIcon: OrganizationLogo(image: WalletMockData.organization.logo, size: kMenuItemLargeIconSize),
            label: const Text('Label'),
            subtitle: const Text('Subtitle'),
            largeIcon: true,
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.light,
        );
        await screenMatchesGolden('menu_item/large.logo.label.subtitle.light');
      },
    );

    testGoldens(
      'LeftIcon, Label, Subtitle - Dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          MenuItem(
            leftIcon: const Icon(Icons.history),
            label: const Text('Label'),
            subtitle: const Text('Subtitle'),
            onPressed: () {},
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('menu_item/icon.label.subtitle.dark');
      },
    );

    testGoldens(
      'Image, Long Label, Long Subtitle, Long underline - Dark',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          MenuItem(
            leftIcon: Image.asset(WalletAssets.logo_ecommerce),
            label: const Text('Lorem ipsum dolor sit amet, consectetur adipiscing elit'),
            subtitle: const Text(
              'Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.',
            ),
            underline: const Text(
              'Quis autem vel eum iure reprehenderit qui in ea voluptate velit esse quam nihil molestiae consequatur',
            ),
            onPressed: () {},
          ),
          surfaceSize: Size(kGoldenSize.width, 204),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('menu_item/image.long_label.long_subtitle.long_underline.dark');
      },
    );

    testGoldens(
      'LeftIcon, Label, Subtitle, Underline - Light - Focused',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          MenuItem(
            leftIcon: const Icon(Icons.history),
            label: const Text('Label'),
            subtitle: const Text('Subtitle'),
            underline: const Text('Underline'),
            onPressed: () {},
          ),
          surfaceSize: Size(kGoldenSize.width, 90),
          brightness: Brightness.light,
        );
        await tester.sendKeyEvent(LogicalKeyboardKey.tab); // Trigger focused state
        await tester.pumpAndSettle();
        await screenMatchesGolden('menu_item/icon.label.subtitle.underline.light.focused');
      },
    );

    testGoldens(
      'LeftIcon, Label, Subtitle, Underline - Dark - Focused',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          MenuItem(
            leftIcon: const Icon(Icons.details_outlined),
            label: const Text('Label'),
            subtitle: const Text('Subtitle'),
            underline: const Text('Underline'),
            onPressed: () {},
          ),
          surfaceSize: Size(kGoldenSize.width, 90),
          brightness: Brightness.dark,
        );
        await tester.sendKeyEvent(LogicalKeyboardKey.tab); // Trigger focused state
        await tester.pumpAndSettle();
        await screenMatchesGolden('menu_item/icon.label.subtitle.underline.dark.focused');
      },
    );

    testGoldens(
      'Multi item (verify vertical growth)',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              MenuItem(
                leftIcon: Container(color: Colors.blueAccent),
                rightIcon: const Icon(Icons.play_arrow_outlined),
                label: const Text('Lorem ipsum dolor sit amet, consectetur adipiscing elit'),
                onPressed: () {},
                dividerSide: DividerSide.top,
              ),
              MenuItem(
                leftIcon: Container(color: Colors.amber),
                rightIcon: const ColoredBox(
                  color: Colors.orange,
                  child: Icon(Icons.add_card_outlined),
                ),
                largeIcon: true,
                errorIcon: Container(color: Colors.red),
                label: const Text('Lorem ipsum dolor sit amet, consectetur adipiscing elit'),
                subtitle: const Text(
                  'Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.',
                ),
                underline: const Text(
                  'Quis autem vel eum iure reprehenderit qui in ea voluptate velit esse quam nihil molestiae consequatur',
                ),
                onPressed: () {},
                dividerSide: DividerSide.both,
              ),
            ],
          ),
          surfaceSize: Size(kGoldenSize.width, 345),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('menu_item/multi_item.dark');
      },
    );
  });
}
