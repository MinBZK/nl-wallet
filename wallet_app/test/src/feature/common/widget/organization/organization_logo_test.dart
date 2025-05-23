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

    testGoldens(
      'landscape logo',
      (tester) async {
        const landscapeSvg = '''
          <svg width="400px" height="100px" viewBox="0 0 400 100" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
            <title>traffic</title>
            <g id="traffic" stroke="none" stroke-width="1" fill="none" fill-rule="evenodd">
              <circle id="Oval" fill="#FF991C" cx="200" cy="50" r="33"></circle>
              <circle id="Oval" fill="#35AD24" cx="80" cy="50" r="33"></circle>
              <circle id="Oval" fill="#FF0000" cx="320" cy="50" r="33"></circle>
            </g>
          </svg>
        ''';
        await tester.pumpWidgetWithAppWrapper(
          OrganizationLogo(
            image: SvgImage(landscapeSvg),
            size: kGoldenSize.height,
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('organization_logo/landscape');
      },
    );

    testGoldens(
      'portrait logo',
      (tester) async {
        final amsterdamPortraitSvg = '''
          <svg id="Amsterdam" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 186.3 633.7" width="735" height="2500">
            <style>.st0{fill:red}</style>
            <path class="st0" d="M0 596.5l55.9-55.9L0 484.7l37.3-37.3 55.9 55.9 55.9-55.9 37.3 37.3-55.9 55.9 55.9 55.9-37.4 37.2-55.9-55.9-55.9 55.9L0 596.5zM0 149l55.9-55.9L0 37.3 37.3 0l55.9 55.9L149 0l37.3 37.3-55.9 55.9 55.9 55.9-37.3 37.2-55.9-55.9-55.9 55.9L0 149zM0 372.9L55.9 317 0 261.2l37.3-37.3 55.9 55.9 55.9-55.9 37.3 37.3-55.9 55.9 55.9 55.9-37.4 37.2-55.9-55.9-55.9 55.9L0 372.9z"/>
          </svg>''';
        await tester.pumpWidgetWithAppWrapper(
          OrganizationLogo(
            image: SvgImage(amsterdamPortraitSvg),
            size: kGoldenSize.height,
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden('organization_logo/portrait');
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
