import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/cupertino.dart';
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/pin/pin_page.dart';

import '../../../wallet_app_test_widget.dart';

class MockPinBloc extends MockBloc<PinEvent, PinState> implements PinBloc {}

/// Tests if the different supported states of the [PinPage] are rendered correctly.
/// Note that the [PinPage] is not a "Screen" and is thus expected to be nested in
/// a [Scaffold] elsewhere. As such not testing brightness scenarios here, since
/// the [PinPage] itself has a transparent background by design.
void main() {
  DeviceBuilder deviceBuilder(WidgetTester tester) {
    return DeviceBuilder()
      ..overrideDevicesForAllScenarios(devices: [Device.phone])
      ..addScenario(
        widget: const PinPage().withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinEntryInProgress(0),
        ),
        name: 'pin_page_initial',
      )
      ..addScenario(
        widget: const PinPage().withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinEntryInProgress(3),
        ),
        name: 'pin_page_progress_3',
      )
      ..addScenario(
        widget: const PinPage().withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinValidateInProgress(),
        ),
        name: 'pin_page_loading',
      )
      ..addScenario(
        widget: const PinPage().withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinValidateFailure(leftoverAttempts: 3, isFinalAttempt: false),
        ),
        name: 'pin_page_error_3_attempts_left',
      );
  }

  group('Golden Tests', () {
    testGoldens('Accessibility Light Test', (tester) async {
      await tester.pumpDeviceBuilder(
        deviceBuilder(tester),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'pin_page_states');
    });
  });

  testWidgets('PinPage renders the correct header', (tester) async {
    final locale = await AppLocalizations.delegate.load(const Locale('en'));
    await tester.pumpWidget(
      WalletAppTestWidget(
        child: const PinPage().withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinEntryInProgress(3),
        ),
      ),
    );

    // Setup finders
    final headerFinder = find.text(locale.pinScreenHeader);

    // Verify all expected widgets show up once
    expect(headerFinder, findsOneWidget);
  });

  testWidgets('PinPage renders the correct header in portrait', (tester) async {
    tester.view.physicalSize = tester.view.physicalSize.flipped;
    addTearDown(() => tester.view.resetPhysicalSize());

    final locale = await AppLocalizations.delegate.load(const Locale('en'));
    await tester.pumpWidget(
      WalletAppTestWidget(
        child: const PinPage().withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinEntryInProgress(3),
        ),
      ),
    );

    // Setup finders
    final headerFinder = find.text(locale.pinScreenHeader);

    // Verify all expected widgets show up once
    expect(headerFinder, findsOneWidget);
  });

  testWidgets('PinPage renders the default error with the correct amount of leftover attempts', (tester) async {
    final locale = await AppLocalizations.delegate.load(const Locale('en'));
    await tester.pumpWidget(
      WalletAppTestWidget(
        child: const PinPage().withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinValidateFailure(leftoverAttempts: 3, isFinalAttempt: false),
        ),
      ),
    );

    // Setup finders
    final headerFinder = find.text(locale.pinScreenErrorHeader);
    final attemptsLeftFinder = find.text(locale.pinScreenAttemptsCount(3));

    // Verify all expected widgets show up once
    expect(headerFinder, findsOneWidget);
    expect(attemptsLeftFinder, findsOneWidget);
  });
}
