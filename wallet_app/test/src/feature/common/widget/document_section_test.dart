import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/common/widget/document_section.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';

void main() {
  const kGoldenSize = Size(350, 128);

  group('goldens', () {
    testGoldens(
      'light document section',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          DocumentSection(
            document: WalletMockData.document,
            organization: WalletMockData.organization,
          ),
          surfaceSize: kGoldenSize,
        );
        await screenMatchesGolden(tester, 'document_section/light');
      },
    );
    testGoldens(
      'dark document section',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          DocumentSection(
            document: WalletMockData.document,
            organization: WalletMockData.organization,
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await screenMatchesGolden(tester, 'document_section/dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('widgets are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        DocumentSection(
          document: WalletMockData.document,
          organization: WalletMockData.organization,
        ),
      );

      // Validate that the widget exists
      final titleFinder = find.text('Title');
      final orgNameFinder = find.text(WalletMockData.organization.displayName.testValue);
      final fileFinder = find.text(WalletMockData.document.fileName);
      expect(titleFinder, findsOneWidget);
      expect(orgNameFinder, findsOneWidget);
      expect(fileFinder, findsOneWidget);
    });
  });
}
