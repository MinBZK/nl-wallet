import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/app_image_data.dart';
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
            image: WalletMockData.organization.logo,
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
          image: WalletMockData.organization.logo,
          size: kGoldenSize.height,
        ),
      );

      // Validate that the widget exists
      expect(WalletMockData.organization.logo, isA<AppAssetImage>(), reason: 'We rely on an AssetImage for this test');
      final widgetFinder = find.image(AssetImage((WalletMockData.organization.logo as AppAssetImage).data));
      expect(widgetFinder, findsOneWidget);
    });
  });
}
