import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/page/terminal_page.dart';
import 'package:wallet/src/feature/common/widget/button/primary_button.dart';
import 'package:wallet/src/feature/common/widget/button/tertiary_button.dart';
import 'package:wallet/src/feature/common/widget/page_illustration.dart';
import 'package:wallet/src/wallet_assets.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('TerminalScreen - general error (uses old golden from .legacy() factory)', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        TerminalPage(
          title: 'Title',
          description: 'Description',
          illustration: const PageIllustration(asset: WalletAssets.svg_digid),
          primaryButton: PrimaryButton(
            text: const Text('PRIMARY'),
            onPressed: () {},
            key: const Key('primaryButtonCta'),
          ),
          secondaryButton: TertiaryButton(
            text: const Text('SECONDARY'),
            icon: const Icon(Icons.arrow_outward_rounded),
            onPressed: () {},
            key: const Key('secondaryButtonCta'),
          ),
        ),
      );
      await screenMatchesGolden('terminal/legacy');
    });

    testGoldens('TerminalPage - 0 buttons', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TerminalPage(
          title: 'Title',
          description: 'Description',
          illustration: PageIllustration(asset: WalletAssets.svg_digid),
        ),
      );
      await screenMatchesGolden('terminal/0_buttons');
    });

    testGoldens('TerminalPage - 1 button', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        TerminalPage(
          title: 'Title',
          description: 'Description',
          illustration: const PageIllustration(asset: WalletAssets.svg_digid),
          primaryButton: PrimaryButton(
            text: const Text('PRIMARY'),
            onPressed: () {},
          ),
        ),
      );
      await screenMatchesGolden('terminal/1_button');
    });

    testGoldens('TerminalPage - 2 buttons', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        TerminalPage(
          title: 'Title',
          description: 'Description',
          illustration: const PageIllustration(asset: WalletAssets.svg_digid),
          primaryButton: PrimaryButton(
            text: const Text('PRIMARY'),
            onPressed: () {},
          ),
          secondaryButton: TertiaryButton(
            text: const Text('SECONDARY'),
            onPressed: () {},
          ),
        ),
      );
      await screenMatchesGolden('terminal/2_buttons');
    });

    testGoldens('TerminalPage - 2 buttons - horizontal', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        TerminalPage(
          title: 'Title',
          description: 'Description',
          illustration: const PageIllustration(asset: WalletAssets.svg_digid),
          primaryButton: PrimaryButton(
            text: const Text('PRIMARY'),
            onPressed: () {},
          ),
          secondaryButton: TertiaryButton(
            text: const Text('SECONDARY'),
            onPressed: () {},
          ),
          preferVerticalButtonLayout: false,
        ),
      );
      await screenMatchesGolden('terminal/2_buttons_horizontal');
    });

    testGoldens('TerminalPage - 2 buttons - horizontal', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        TerminalPage(
          title: 'Title',
          description: 'Description',
          illustration: const PageIllustration(asset: WalletAssets.svg_digid),
          primaryButton: PrimaryButton(
            text: const Text('PRIMARY'),
            onPressed: () {},
          ),
          secondaryButton: TertiaryButton(
            text: const Text('SECONDARY'),
            onPressed: () {},
          ),
        ),
        brightness: .dark,
        surfaceSize: iphoneXSizeLandscape,
      );
      await screenMatchesGolden('terminal/2_buttons.dark.landscape');
    });
  });
}
