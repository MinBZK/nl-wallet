import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/pin/pin_overlay.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.dart';

void main() {
  testWidgets('verify PinOverlay shows child when status is unlocked', (tester) async {
    await tester.pumpWidget(
      WalletAppTestWidget(
        child: PinOverlay(
          bloc: PinBloc(Mocks.create()),
          isLockedStream: Stream.value(false),
          child: const Text('unlocked'),
        ),
      ),
    );

    // Make sure stream is processed
    await tester.pumpAndSettle();

    // Setup finders
    final titleFinder = find.text('unlocked');

    // Verify all expected widgets show up once
    expect(titleFinder, findsOneWidget);
  });

  testWidgets('verify PinOverlay hides child when status is locked', (tester) async {
    await tester.pumpWidget(
      WalletAppTestWidget(
        child: PinOverlay(
          bloc: PinBloc(Mocks.create()),
          isLockedStream: Stream.value(true),
          child: const Text('locked'),
        ),
      ),
    );

    // Make sure stream is processed
    await tester.pumpAndSettle();

    // Setup finders
    final titleFinder = find.text('locked');

    // Verify the locked widget is NOT shown
    expect(titleFinder, findsNothing);
  });
}
