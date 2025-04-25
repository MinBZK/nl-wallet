import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/app_image_data.dart';
import 'package:wallet/src/feature/common/widget/organization/organization_logo.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mock_data.dart';
import '../../../../test_util/golden_utils.dart';

void main() {
  const kGoldenSize = Size(80, 80);

  group('goldens', () {
    testGoldens(
      'rijks logo',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          OrganizationLogo(
            image: WalletMockData.organization.logo,
            size: kGoldenSize.height,
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('organization_logo/rijks');
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
      final widgetFinder = find.image(AssetImage((WalletMockData.organization.logo as AppAssetImage).name));
      expect(widgetFinder, findsOneWidget);
    });
  });
}
