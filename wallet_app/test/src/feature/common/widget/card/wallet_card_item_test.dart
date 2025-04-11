import 'dart:ui';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/common/widget/card/card_holograph.dart';
import 'package:wallet/src/feature/common/widget/card/card_logo.dart';
import 'package:wallet/src/feature/common/widget/card/mock_card_background.dart';
import 'package:wallet/src/feature/common/widget/card/wallet_card_item.dart';
import 'package:wallet/src/feature/common/widget/svg_or_image.dart';
import 'package:wallet/src/theme/dark_wallet_theme.dart';
import 'package:wallet/src/theme/light_wallet_theme.dart';
import 'package:wallet/src/wallet_assets.dart';
import 'package:wallet_core/core.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mock_data.dart';
import '../../../../test_util/golden_utils.dart';
import '../../../../test_util/test_utils.dart';

void _voidCallback() {}

/// This test also (indirectly) verifies:
/// [ShowDetailsCta], [CardLogo] and [CardHolograph]
void main() {
  setUp(TestUtils.mockSensorsPlugin);

  group('goldens', () {
    testGoldens(
      'Card with simple rendering is rendered as expected',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          surfaceSize: const Size(328, 198),
          Builder(
            builder: (context) {
              return WalletCardItem.fromWalletCard(context, WalletMockData.simpleRenderingCard);
            },
          ),
        );
        await screenMatchesGolden('wallet_card_item/simple_rendering');
      },
    );

    testGoldens(
      'Cards adapt based on provided brightness',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          Column(
            spacing: 16,
            children: [
              WalletCardItem(
                title: 'Dark Card',
                subtitle: 'subtitle',
                background: MockCardBackground(docType: kAddressDocType),
                logo: CardLogo(logo: WalletAssets.logo_card_rijksoverheid),
                textColor: DarkWalletTheme.textColor,
                onPressed: _voidCallback,
              ),
              WalletCardItem(
                title: 'Light Card',
                subtitle: 'subtitle',
                background: MockCardBackground(docType: kPidDocType),
                logo: CardLogo(logo: WalletAssets.logo_card_rijksoverheid),
                textColor: LightWalletTheme.textColor,
                onPressed: _voidCallback,
              ),
            ],
          ),
          surfaceSize: const Size(330, 406),
        );
        await screenMatchesGolden('wallet_card_item/brightness');
      },
    );

    /// The max 50 char limit is not enforced in code. It's simply
    /// the maximum length we are currently using to verify the UI
    /// and is thus the max length we currently guarantee to support.
    testGoldens(
      'Card scales vertically with content',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          Column(
            spacing: 16,
            children: [
              WalletCardItem(
                title: '50 characters looooooong title is consider the max',
                subtitle: '50 characters loong subtitle is considered the max',
                background: MockCardBackground(docType: kAddressDocType),
                logo: CardLogo(logo: WalletAssets.logo_card_rijksoverheid),
                textColor: DarkWalletTheme.textColor,
                onPressed: _voidCallback,
              ),
              MediaQuery(
                data: MediaQueryData(textScaler: TextScaler.linear(3.2)),
                child: WalletCardItem(
                  title: '50 characters looooooong title is consider the max',
                  subtitle: '50 characters loong subtitle is considered the max',
                  background: MockCardBackground(docType: kPidDocType),
                  logo: CardLogo(logo: WalletAssets.logo_card_rijksoverheid),
                  textColor: LightWalletTheme.textColor,
                  onPressed: _voidCallback,
                ),
              ),
            ],
          ),
          surfaceSize: const Size(330, 1584),
        );
        await screenMatchesGolden('wallet_card_item/scaling');
      },
    );

    testGoldens(
      'Subtitles are rendered when provided',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          Column(
            spacing: 16,
            children: [
              WalletCardItem(
                title: 'TITLE',
                subtitle: 'SUBTITLE',
                textColor: LightWalletTheme.textColor,
                background: MockCardBackground(docType: kPidDocType),
                logo: CardLogo(logo: WalletAssets.logo_card_rijksoverheid),
                onPressed: _voidCallback,
              ),
              WalletCardItem(
                title: 'TITLE',
                subtitle: 'SUBTITLE',
                textColor: LightWalletTheme.textColor,
                onPressed: _voidCallback,
                background: MockCardBackground(docType: kPidDocType),
              ),
              WalletCardItem(
                title: 'TITLE - NO SUBTITLE',
                textColor: LightWalletTheme.textColor,
                background: MockCardBackground(docType: kPidDocType),
                logo: CardLogo(logo: WalletAssets.logo_card_rijksoverheid),
                onPressed: _voidCallback,
              ),
              WalletCardItem(
                title: 'TITLE - NO SHOW DETAILS',
                textColor: LightWalletTheme.textColor,
                background: MockCardBackground(docType: kPidDocType),
                logo: CardLogo(logo: WalletAssets.logo_card_rijksoverheid),
              ),
            ],
          ),
          surfaceSize: const Size(330, 824),
        );

        await screenMatchesGolden('wallet_card_item/content');
      },
    );

    testGoldens(
      'Verify holograph is rendered as expected',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          WalletCardItem(
            title: 'Holograph',
            textColor: DarkWalletTheme.textColor,
            background: MockCardBackground(docType: kAddressDocType),
            logo: CardLogo(logo: WalletAssets.logo_card_rijksoverheid),
            holograph: CardHolograph(holograph: WalletAssets.svg_rijks_card_holo, brightness: Brightness.dark),
          ),
          surfaceSize: const Size(328, 198),
        );

        await screenMatchesGolden('wallet_card_item/holograph');
      },
    );
  });

  group('widgets', () {
    testWidgets('verify title & subtitle are shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        WalletCardItem(
          title: 'title',
          background: const SvgOrImage(asset: WalletAssets.svg_rijks_card_bg_dark),
          onPressed: () {},
          ctaAnimation: CtaAnimation.visible,
          logo: const SvgOrImage(asset: WalletAssets.logo_card_rijksoverheid),
          subtitle: 'subtitle1',
        ),
      );
      expect(find.text('title'), findsOneWidget);
      expect(find.text('subtitle1'), findsOneWidget);
    });

    testWidgets('verify title, subtitle are shown in shuttle card', (tester) async {
      final testCard = WalletMockData.simpleRenderingCard;
      await tester.pumpWidgetWithAppWrapper(
        WalletCardItem.buildShuttleCard(
          const AlwaysStoppedAnimation(0),
          testCard,
        ),
      );
      expect(find.text(testCard.metadata.first.name), findsOneWidget);
      expect(find.text(testCard.metadata.first.rawSummary ?? ''), findsOneWidget);
    });
  });
}
