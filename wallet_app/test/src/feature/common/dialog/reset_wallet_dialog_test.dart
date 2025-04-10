import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/wallet/reset_wallet_usecase.dart';
import 'package:wallet/src/feature/common/dialog/reset_wallet_dialog.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.mocks.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

void main() {
  testWidgets('ResetWalletDialog shows expected copy', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      const ResetWalletDialog(),
    );

    final l10n = await TestUtils.englishLocalizations;

    // Setup finders
    final titleFinder = find.text(l10n.resetWalletDialogTitle, findRichText: true);
    final descriptionFinder = find.text(l10n.resetWalletDialogBody, findRichText: true);
    final cancelCtaFinder = find.text(l10n.resetWalletDialogCancelCta.toUpperCase(), findRichText: true);
    final confirmCtaFinder = find.text(l10n.resetWalletDialogConfirmCta.toUpperCase(), findRichText: true);

    // Verify all expected widgets show up once
    expect(titleFinder, findsOneWidget);
    expect(descriptionFinder, findsOneWidget);
    expect(cancelCtaFinder, findsOneWidget);
    expect(confirmCtaFinder, findsOneWidget);
  });

  testWidgets('ResetWalletDialog invokes ResetWalletUseCase when confirm is pressed', (tester) async {
    final ResetWalletUseCase usecase = MockResetWalletUseCase();
    await tester.pumpWidgetWithAppWrapper(
      const ResetWalletDialog(),
      providers: [
        RepositoryProvider<ResetWalletUseCase>(create: (BuildContext context) => usecase),
      ],
    );

    final l10n = await TestUtils.englishLocalizations;
    final buttonFinder = find.text(l10n.resetWalletDialogConfirmCta.toUpperCase(), findRichText: true);
    expect(buttonFinder, findsOneWidget);

    await tester.tap(buttonFinder);

    verify(usecase.invoke()).called(1);
  });

  testWidgets('ResetWalletDialog does not invoke ResetWalletUseCase when cancel is pressed', (tester) async {
    final ResetWalletUseCase usecase = MockResetWalletUseCase();
    await tester.pumpWidgetWithAppWrapper(
      const ResetWalletDialog(),
      providers: [
        RepositoryProvider<ResetWalletUseCase>(create: (BuildContext context) => usecase),
      ],
    );

    final l10n = await TestUtils.englishLocalizations;
    final buttonFinder = find.text(l10n.resetWalletDialogCancelCta.toUpperCase());
    expect(buttonFinder, findsOneWidget);

    await tester.tap(buttonFinder);

    verifyNever(usecase.invoke());
  });

  group('goldens', () {
    testGoldens(
      'Reset Wallet Dialog',
      (tester) async {
        final Key showDialogButton = Key('showDialogButton');
        await tester.pumpWidgetWithAppWrapper(
          Scaffold(
            body: Builder(
              builder: (context) {
                return Center(
                  child: TextButton(
                    onPressed: () => ResetWalletDialog.show(context),
                    child: Text('Show Dialog', key: showDialogButton),
                  ),
                );
              },
            ),
          ),
        );
        await tester.tap(find.byKey(showDialogButton));
        await tester.pumpAndSettle();
        await screenMatchesGolden('reset_dialog');
      },
    );
  });
}
