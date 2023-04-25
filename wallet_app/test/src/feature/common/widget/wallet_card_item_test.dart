import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/widget/card/wallet_card_item.dart';

import '../../../../wallet_app_test_widget.dart';

const _kDarkBg = 'assets/svg/rijks_card_bg_dark.svg';
const _kLightBg = 'assets/svg/rijks_card_bg_light.svg';
const _kLogo = 'assets/non-free/images/logo_card_rijksoverheid.png';
const _kHolo = 'assets/svg/rijks_card_holo.svg';

void _voidCallback() {}

void main() {
  setUp(() {
    const MethodChannel('dev.fluttercommunity.plus/sensors/accelerometer')
        .setMockMethodCallHandler((MethodCall methodCall) async {
      if (methodCall.method == 'listen') {
        return <String, dynamic>{};
      }
      return null;
    });
  });

  group('Golden Tests', () {
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
              background: _kDarkBg,
              logo: _kLogo,
              onPressed: _voidCallback,
            ),
          )
          ..addScenario(
            'light',
            const WalletCardItem(
              title: 'Light Card',
              subtitle1: 'subtitle',
              subtitle2: 'subtitle2',
              holograph: _kHolo,
              brightness: Brightness.light,
              background: _kLightBg,
              logo: _kLogo,
              onPressed: _voidCallback,
            ),
          );
        await tester.pumpWidgetBuilder(
          builder.build(),
          surfaceSize: const Size(400, 556),
          wrapper: walletAppWrapper(),
        );
        await screenMatchesGolden(tester, 'wallet_card_item/brightness');
      },
    );

    // FIXME: The max 50 char limit is not enforced in code. It's simply
    // FIXME: the maximum length we are currently using to verify the UI
    // FIXME: and is thus the max length we currently guarantee to support.
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
              holograph: _kHolo,
              brightness: Brightness.light,
              background: _kLightBg,
              logo: _kLogo,
              onPressed: _voidCallback,
            ),
          )
          ..addTextScaleScenario(
            'maximum textScaling',
            const WalletCardItem(
              title: '50 characters looooooong title is consider the max',
              subtitle1: '50 characters loong subtitle is considered the max',
              subtitle2: '50 characters long subtitle2 is considered the max',
              holograph: _kHolo,
              brightness: Brightness.light,
              background: _kLightBg,
              logo: _kLogo,
              onPressed: _voidCallback,
            ),
          );

        await tester.pumpWidgetBuilder(
          builder.build(),
          surfaceSize: const Size(400, 1320),
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
              background: _kLightBg,
              logo: _kLogo,
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
              background: _kLightBg,
              onPressed: _voidCallback,
            ),
          )
          ..addScenario(
            'no subtitle 2',
            const WalletCardItem(
              title: 'TITLE - NO SUBTITLE 2',
              subtitle1: 'SUBTITLE',
              brightness: Brightness.light,
              background: _kLightBg,
              logo: _kLogo,
              onPressed: _voidCallback,
            ),
          )
          ..addScenario(
            'no subtitle',
            const WalletCardItem(
              title: 'TITLE - NO SUBTITLE',
              subtitle2: 'SUBTITLE - 2',
              brightness: Brightness.light,
              background: _kLightBg,
              logo: _kLogo,
              onPressed: _voidCallback,
            ),
          )
          ..addScenario(
            'no subtitles',
            const WalletCardItem(
              title: 'TITLE - NO SUBTITLES',
              brightness: Brightness.light,
              background: _kLightBg,
              logo: _kLogo,
              onPressed: _voidCallback,
            ),
          )
          ..addScenario(
            'no show details',
            const WalletCardItem(
              title: 'TITLE - NO SHOW DETAILS',
              brightness: Brightness.light,
              background: _kLightBg,
              logo: _kLogo,
            ),
          );

        await tester.pumpWidgetBuilder(
          builder.build(),
          surfaceSize: const Size(400, 1638),
          wrapper: walletAppWrapper(),
        );
        await screenMatchesGolden(tester, 'wallet_card_item/content');
      },
    );
  });
}
