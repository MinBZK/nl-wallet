import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/common/widget/organization/organization_logo.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/mock_data.dart';

void main() {
  const kGoldenSize = Size(80, 80);

  group('goldens', () {
    testGoldens(
      'light text',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          OrganizationLogo(
            image: AssetImage(WalletMockData.organization.logoUrl),
            size: kGoldenSize.height,
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'organization_logo/light');
      },
    );
  });

  group('widgets', () {
    testWidgets('image is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        OrganizationLogo(
          image: AssetImage(WalletMockData.organization.logoUrl),
          size: kGoldenSize.height,
        ),
      );

      // Validate that the widget exists
      final widgetFinder = find.image(AssetImage(WalletMockData.organization.logoUrl));
      expect(widgetFinder, findsOneWidget);
    });
  });
}
