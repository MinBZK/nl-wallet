import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/common/widget/card/wallet_card_item.dart';
import 'package:wallet/src/wallet_assets.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mock_data.dart';
import '../../../../util/test_utils.dart';

void _voidCallback() {}

/// This test also (indirectly) verifies:
/// [ShowDetailsCta], [CardLogo] and [CardHolograph]
void main() {
  setUp(TestUtils.mockSensorsPlugin);

  group('goldens', () {
    testGoldens(
      'Cards adapt based on provided brightness',
      (tester) async {
        final builder = GoldenBuilder.column()
          ..addScenario(
            'dark',
            const WalletCardItem(
              title: 'Dark Card',
              subtitle1: 'subtitle',
              subtitle2: 'subtitle2',
              brightness: Brightness.dark,
              background: WalletAssets.svg_rijks_card_bg_dark,
              logo: WalletAssets.logo_card_rijksoverheid,
              onPressed: _voidCallback,
            ),
          )
          ..addScenario(
            'light',
            const WalletCardItem(
              title: 'Light Card',
              subtitle1: 'subtitle',
              subtitle2: 'subtitle2',
              holograph: WalletAssets.svg_rijks_card_holo,
              brightness: Brightness.light,
              background: WalletAssets.svg_rijks_card_bg_light,
              logo: WalletAssets.logo_card_rijksoverheid,
              onPressed: _voidCallback,
            ),
          );
        await tester.pumpWidgetBuilder(
          builder.build(),
          surfaceSize: const Size(344, 490),
          wrapper: walletAppWrapper(),
        );
        await screenMatchesGolden(tester, 'wallet_card_item/brightness');
      },
    );

    /// The max 50 char limit is not enforced in code. It's simply
    /// the maximum length we are currently using to verify the UI
    /// and is thus the max length we currently guarantee to support.
    testGoldens(
      'Card scales vertically with content',
      (tester) async {
        final builder = GoldenBuilder.column()
          ..addScenario(
            'base case',
            const WalletCardItem(
              title: '50 characters looooooong title is consider the max',
              subtitle1: '50 characters loong subtitle is considered the max',
              subtitle2: '50 characters long subtitle2 is considered the max',
              holograph: WalletAssets.svg_rijks_card_holo,
              brightness: Brightness.light,
              background: WalletAssets.svg_rijks_card_bg_light,
              logo: WalletAssets.logo_card_rijksoverheid,
              onPressed: _voidCallback,
            ),
          )
          ..addTextScaleScenario(
            'maximum textScaling',
            const WalletCardItem(
              title: '50 characters looooooong title is consider the max',
              subtitle1: '50 characters loong subtitle is considered the max',
              subtitle2: '50 characters long subtitle2 is considered the max',
              holograph: WalletAssets.svg_rijks_card_holo,
              brightness: Brightness.light,
              background: WalletAssets.svg_rijks_card_bg_light,
              logo: WalletAssets.logo_card_rijksoverheid,
              onPressed: _voidCallback,
            ),
          );

        await tester.pumpWidgetBuilder(
          builder.build(),
          surfaceSize: const Size(344, 2135),
          wrapper: walletAppWrapper(),
        );
        await screenMatchesGolden(tester, 'wallet_card_item/scaling');
      },
    );

    testGoldens(
      'Subtitles are rendered when provided',
      (tester) async {
        final builder = GoldenBuilder.column()
          ..addScenario(
            'base case',
            const WalletCardItem(
              title: 'TITLE',
              subtitle1: 'SUBTITLE',
              subtitle2: 'SUBTITLE - 2',
              brightness: Brightness.light,
              background: WalletAssets.svg_rijks_card_bg_light,
              logo: WalletAssets.logo_card_rijksoverheid,
              onPressed: _voidCallback,
            ),
          )
          ..addScenario(
            'no logo',
            const WalletCardItem(
              title: 'TITLE',
              subtitle1: 'SUBTITLE',
              subtitle2: 'SUBTITLE - 2',
              brightness: Brightness.light,
              background: WalletAssets.svg_rijks_card_bg_light,
              onPressed: _voidCallback,
            ),
          )
          ..addScenario(
            'no subtitle 2',
            const WalletCardItem(
              title: 'TITLE - NO SUBTITLE 2',
              subtitle1: 'SUBTITLE',
              brightness: Brightness.light,
              background: WalletAssets.svg_rijks_card_bg_light,
              logo: WalletAssets.logo_card_rijksoverheid,
              onPressed: _voidCallback,
            ),
          )
          ..addScenario(
            'no subtitle',
            const WalletCardItem(
              title: 'TITLE - NO SUBTITLE',
              subtitle2: 'SUBTITLE - 2',
              brightness: Brightness.light,
              background: WalletAssets.svg_rijks_card_bg_light,
              logo: WalletAssets.logo_card_rijksoverheid,
              onPressed: _voidCallback,
            ),
          )
          ..addScenario(
            'no subtitles',
            const WalletCardItem(
              title: 'TITLE - NO SUBTITLES',
              brightness: Brightness.light,
              background: WalletAssets.svg_rijks_card_bg_light,
              logo: WalletAssets.logo_card_rijksoverheid,
              onPressed: _voidCallback,
            ),
          )
          ..addScenario(
            'no show details',
            const WalletCardItem(
              title: 'TITLE - NO SHOW DETAILS',
              brightness: Brightness.light,
              background: WalletAssets.svg_rijks_card_bg_light,
              logo: WalletAssets.logo_card_rijksoverheid,
            ),
          );

        await tester.pumpWidgetBuilder(
          builder.build(),
          surfaceSize: const Size(344, 1452),
          wrapper: walletAppWrapper(),
        );
        await screenMatchesGolden(tester, 'wallet_card_item/content');
      },
    );
  });

  group('widgets', () {
    testWidgets('verify title, subtitle, subtitle2 are shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        WalletCardItem(
          title: 'title',
          background: WalletAssets.svg_rijks_card_bg_dark,
          brightness: Brightness.light,
          onPressed: () {},
          ctaAnimation: CtaAnimation.visible,
          holograph: WalletAssets.svg_rijks_card_holo,
          logo: WalletAssets.logo_card_rijksoverheid,
          subtitle1: 'subtitle1',
          subtitle2: 'subtitle2',
        ),
      );
      expect(find.text('title'), findsOneWidget);
      expect(find.text('subtitle1'), findsOneWidget);
      expect(find.text('subtitle2'), findsOneWidget);
    });

    testWidgets('verify title, subtitle are shown in shuttle card', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        WalletCardItem.buildShuttleCard(
          const AlwaysStoppedAnimation(0),
          WalletMockData.cardFront,
        ),
      );
      expect(find.text(WalletMockData.cardFront.title.testValue), findsOneWidget);
      expect(find.text(WalletMockData.cardFront.subtitle!.testValue), findsOneWidget);
    });
  });
}
