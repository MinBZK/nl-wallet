import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/environment.dart';
import 'package:wallet/src/feature/developer_settings/bloc/developer_settings_bloc.dart';
import 'package:wallet/src/feature/developer_settings/developer_settings_screen.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.dart';
import '../../test_util/test_utils.dart';

class MockDeveloperSettingsBloc extends MockBloc<DeveloperSettingsEvent, DeveloperSettingsState>
    implements DeveloperSettingsBloc {}

void main() {
  testWidgets('developer setting follows the build flag and saves changes', (tester) async {
    final repository = MockCloseProximityRepository();
    when(repository.getWrprcStrictMode()).thenAnswer((_) async => true);
    expect(
      WalletRoutes.allRoutes.contains(WalletRoutes.developerSettingsRoute),
      Environment.enableWrprcStrictModeToggle,
    );
    expect(WalletRoutes.publicRoutes, isNot(contains(WalletRoutes.developerSettingsRoute)));

    await tester.pumpWidgetWithAppWrapper(
      BlocProvider<DeveloperSettingsBloc>(
        create: (_) => DeveloperSettingsBloc(repository)..add(const DeveloperSettingsLoadTriggered()),
        child: const DeveloperSettingsScreen(),
      ),
    );
    final toggle = find.byType(Switch);
    if (Environment.enableWrprcStrictModeToggle) {
      expect(tester.widget<Switch>(toggle).value, isTrue);
      await tester.tap(toggle);
      await tester.pumpAndSettle();
      verify(repository.setWrprcStrictMode(enabled: false)).called(1);
      expect(tester.widget<Switch>(toggle).value, isFalse);
    } else {
      expect(toggle, findsNothing);
      verifyZeroInteractions(repository);
    }
  });

  group('enabled developer settings', () {
    const disabledStates = {
      'not yet loaded': DeveloperSettingsState(),
      'loading': DeveloperSettingsState(isBusy: true),
      'saving': DeveloperSettingsState(wrprcStrictMode: false, isBusy: true),
    };

    for (final entry in disabledStates.entries) {
      testWidgets('the switch is disabled when ${entry.key}', (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const DeveloperSettingsScreen().withState<DeveloperSettingsBloc, DeveloperSettingsState>(
            MockDeveloperSettingsBloc(),
            entry.value,
          ),
        );

        final toggle = tester.widget<Switch>(find.byType(Switch));
        expect(toggle.value, entry.value.wrprcStrictMode ?? true);
        expect(toggle.onChanged, isNull);
      });
    }

    for (final confirmedValue in <bool?>[null, true, false]) {
      testWidgets('an error preserves the confirmed value $confirmedValue', (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const DeveloperSettingsScreen().withState<DeveloperSettingsBloc, DeveloperSettingsState>(
            MockDeveloperSettingsBloc(),
            DeveloperSettingsState(wrprcStrictMode: confirmedValue, hasError: true),
          ),
        );

        final toggle = tester.widget<Switch>(find.byType(Switch));
        expect(toggle.value, confirmedValue ?? true);
        expect(toggle.onChanged, confirmedValue == null ? isNull : isNotNull);
        final l10n = await TestUtils.englishLocalizations;
        expect(find.text(l10n.developerSettingsScreenError), findsOneWidget);
      });
    }
  }, skip: !Environment.enableWrprcStrictModeToggle);
}
