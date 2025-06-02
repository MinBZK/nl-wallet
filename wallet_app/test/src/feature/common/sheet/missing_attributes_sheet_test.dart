import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/common/sheet/missing_attributes_sheet.dart';
import 'package:wallet/src/feature/common/widget/button/tertiary_button.dart';
import 'package:wallet/src/feature/common/widget/text/body_text.dart';
import 'package:wallet/src/feature/common/widget/text/title_text.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

const kGoldenSize = Size(350, 291);

void main() {
  group('goldens', () {
    testGoldens(
      'light - single attribute',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          MissingAttributesSheet(
            missingAttributes: [
              MissingAttribute(
                label: {const Locale('en'): 'First Name'},
              ),
            ],
          ),
          surfaceSize: const Size(350, 250),
        );
        await tester.pumpAndSettle();
        await screenMatchesGolden('missing_attributes_sheet/single_attribute.light');
      },
    );

    testGoldens(
      'light - multiple attributes',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          MissingAttributesSheet(
            missingAttributes: [
              MissingAttribute(
                label: {const Locale('en'): 'First Name'},
              ),
              MissingAttribute(
                label: {const Locale('en'): 'Last Name'},
              ),
              MissingAttribute(
                label: {const Locale('en'): 'Date of Birth'},
              ),
            ],
          ),
          surfaceSize: kGoldenSize,
        );
        await tester.pumpAndSettle();
        await screenMatchesGolden('missing_attributes_sheet/multiple_attributes.light');
      },
    );

    testGoldens(
      'dark - multiple attributes',
      (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          MissingAttributesSheet(
            missingAttributes: [
              MissingAttribute(
                label: {const Locale('en'): 'First Name'},
              ),
              MissingAttribute(
                label: {const Locale('en'): 'Last Name'},
              ),
              MissingAttribute(
                label: {const Locale('en'): 'Date of Birth'},
              ),
            ],
          ),
          surfaceSize: kGoldenSize,
          brightness: Brightness.dark,
        );
        await tester.pumpAndSettle();
        await screenMatchesGolden('missing_attributes_sheet/multiple_attributes.dark');
      },
    );
  });

  group('widgets', () {
    testWidgets('displays title and attributes correctly', (tester) async {
      final missingAttributes = [
        MissingAttribute(
          label: {const Locale('en'): 'First Name'},
        ),
        MissingAttribute(
          label: {const Locale('en'): 'Last Name'},
        ),
      ];

      await tester.pumpWidgetWithAppWrapper(
        MissingAttributesSheet(
          missingAttributes: missingAttributes,
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      // Validate that title widgets exist
      final titleWidgets = find.byType(TitleText);
      expect(titleWidgets, findsAtLeastNWidgets(2)); // Main title + attributes title
      expect(find.text(l10n.missingAttributesSheetTitle), findsOneWidget);
      expect(find.text(l10n.missingAttributesSheetAttributesTitle), findsOneWidget);

      // Validate that body text widgets exist for each attribute
      final bodyTextWidgets = find.byType(BodyText);
      expect(bodyTextWidgets, findsNWidgets(missingAttributes.length));

      // Validate that close button exists
      final closeButton = find.byType(TertiaryButton);
      expect(closeButton, findsOneWidget);
      expect(find.text(l10n.generalClose), findsOneWidget);
    });

    testWidgets('close button dismisses sheet', (tester) async {
      bool sheetDismissed = false;

      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) => Scaffold(
            body: ElevatedButton(
              onPressed: () async {
                await MissingAttributesSheet.show(
                  context,
                  [
                    MissingAttribute(
                      label: {const Locale('en'): 'First Name'},
                    ),
                  ],
                );
                sheetDismissed = true;
              },
              child: const Text('Show Sheet'),
            ),
          ),
        ),
      );

      // Open the sheet
      await tester.tap(find.text('Show Sheet'));
      await tester.pumpAndSettle();

      // Verify sheet is displayed
      expect(find.byType(MissingAttributesSheet), findsOneWidget);
      expect(sheetDismissed, false);

      // Tap the close button
      final l10n = await TestUtils.englishLocalizations;
      await tester.tap(find.text(l10n.generalClose));
      await tester.pumpAndSettle();

      // Verify sheet is dismissed
      expect(find.byType(MissingAttributesSheet), findsNothing);
      expect(sheetDismissed, true);
    });

    testWidgets('displays correct number of attributes', (tester) async {
      final missingAttributes = [
        MissingAttribute(label: {const Locale('en'): 'Attribute 1'}),
        MissingAttribute(label: {const Locale('en'): 'Attribute 2'}),
        MissingAttribute(label: {const Locale('en'): 'Attribute 3'}),
        MissingAttribute(label: {const Locale('en'): 'Attribute 4'}),
      ];

      await tester.pumpWidgetWithAppWrapper(
        MissingAttributesSheet(
          missingAttributes: missingAttributes,
        ),
      );

      // Validate that all attributes are displayed
      for (int i = 0; i < missingAttributes.length; i++) {
        expect(find.text('Attribute ${i + 1}'), findsOneWidget);
      }
    });

    testWidgets('can handle empty attributes list', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const MissingAttributesSheet(
          missingAttributes: [],
        ),
      );

      // Should still display titles and close button
      expect(find.byType(TitleText), findsAtLeastNWidgets(2));
      expect(find.byType(TertiaryButton), findsOneWidget);

      // Should not display any attribute body text
      expect(find.byType(BodyText), findsNothing);
    });
  });
}
