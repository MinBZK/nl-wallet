import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/page/check_data_offering_page.dart';
import 'package:wallet/src/feature/common/widget/button/confirm/confirm_buttons.dart';
import 'package:wallet/src/feature/common/widget/button/primary_button.dart';
import 'package:wallet/src/feature/common/widget/button/secondary_button.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';

void main() {
  group('goldens', () {
    testGoldens(
      'light page',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          CheckDataOfferingPage(
            title: 'Title',
            subtitle: 'Subtitle',
            cardFront: WalletMockData.cardFront,
            footerCta: 'Footer CTA',
            overline: 'Overline',
            showHeaderAttributesDivider: true,
            bottomSection: const ConfirmButtons(
              primaryButton: PrimaryButton(
                text: Text('Primary'),
              ),
              secondaryButton: SecondaryButton(
                text: Text('Secondary'),
              ),
            ),
            attributes: [
              WalletMockData.textDataAttribute,
              WalletMockData.textDataAttribute,
              WalletMockData.textDataAttribute,
            ],
          ),
        );
        await screenMatchesGolden(tester, 'check_data_offering_page/light');
      },
    );

    testGoldens(
      'light page - no footer',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          CheckDataOfferingPage(
            title: 'Title',
            subtitle: 'No footer sample page',
            cardFront: WalletMockData.cardFront,
            overline: 'Overline',
            showHeaderAttributesDivider: true,
            bottomSection: const ConfirmButtons(
              primaryButton: PrimaryButton(
                text: Text('Primary'),
              ),
              secondaryButton: SecondaryButton(
                text: Text('Secondary'),
              ),
            ),
            attributes: [
              WalletMockData.textDataAttribute,
              WalletMockData.textDataAttribute,
              WalletMockData.textDataAttribute,
            ],
          ),
        );
        await screenMatchesGolden(tester, 'check_data_offering_page/no_footer.light');
      },
    );

    testGoldens(
      'dark page',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          CheckDataOfferingPage(
            title: 'Title',
            subtitle: 'Subtitle',
            cardFront: WalletMockData.cardFront,
            footerCta: 'Footer CTA',
            overline: 'Overline',
            showHeaderAttributesDivider: true,
            bottomSection: const ConfirmButtons(
              primaryButton: PrimaryButton(
                text: Text('Primary'),
              ),
              secondaryButton: SecondaryButton(
                text: Text('Secondary'),
              ),
            ),
            attributes: [
              WalletMockData.textDataAttribute,
              WalletMockData.textDataAttribute,
              WalletMockData.textDataAttribute,
            ],
          ),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden(tester, 'check_data_offering_page/dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('widgets are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CheckDataOfferingPage(
          title: 'T',
          subtitle: 'S',
          cardFront: WalletMockData.cardFront,
          footerCta: 'F',
          overline: 'O',
          showHeaderAttributesDivider: true,
          bottomSection: const Text('BS'),
          attributes: [WalletMockData.textDataAttribute],
        ),
      );

      // Validate that the widget exists
      final titleFinder = find.text('T');
      final subtitleFinder = find.text('S');
      final overlineFinder = find.text('O');
      final bottomSectionFinder = find.text('BS');
      final valueFinder = find.text('Value');
      final labelFinder = find.text('Label');
      expect(titleFinder, findsOneWidget);
      expect(subtitleFinder, findsOneWidget);
      expect(overlineFinder, findsOneWidget);
      expect(bottomSectionFinder, findsOneWidget);
      expect(valueFinder, findsOneWidget);
      expect(labelFinder, findsOneWidget);
    });

    testWidgets('optional widgets are not visible when unset', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const CheckDataOfferingPage(
          title: 'T',
          showHeaderAttributesDivider: true,
          bottomSection: Text('BS'),
          attributes: [],
        ),
      );

      // Validate that the unset widgets don't exists
      final titleFinder = find.text('T');
      final subtitleFinder = find.text('S');
      final overlineFinder = find.text('O');
      final bottomSectionFinder = find.text('BS');
      final valueFinder = find.text('Value');
      final labelFinder = find.text('Label');
      expect(titleFinder, findsOneWidget);
      expect(subtitleFinder, findsNothing);
      expect(overlineFinder, findsNothing);
      expect(bottomSectionFinder, findsOneWidget);
      expect(valueFinder, findsNothing);
      expect(labelFinder, findsNothing);
    });
  });
}
