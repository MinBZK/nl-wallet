import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/feature/developer_settings/bloc/developer_settings_bloc.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockCloseProximityRepository repository;

  setUp(() {
    repository = MockCloseProximityRepository();
    when(repository.setWrprcStrictMode(enabled: anyNamed('enabled'))).thenAnswer((_) async {});
  });

  group('load', () {
    for (final enabled in [true, false]) {
      blocTest<DeveloperSettingsBloc, DeveloperSettingsState>(
        'loads the persisted strict mode value $enabled',
        setUp: () => when(repository.getWrprcStrictMode()).thenAnswer((_) async => enabled),
        build: () => DeveloperSettingsBloc(repository),
        act: (bloc) => bloc.add(const DeveloperSettingsLoadTriggered()),
        expect: () => [
          const DeveloperSettingsState(isBusy: true),
          DeveloperSettingsState(wrprcStrictMode: enabled),
        ],
      );
    }

    blocTest<DeveloperSettingsBloc, DeveloperSettingsState>(
      'keeps the setting unknown when loading fails',
      setUp: () => when(repository.getWrprcStrictMode()).thenThrow(StateError('load failed')),
      build: () => DeveloperSettingsBloc(repository),
      act: (bloc) => bloc.add(const DeveloperSettingsLoadTriggered()),
      expect: () => [
        const DeveloperSettingsState(isBusy: true),
        const DeveloperSettingsState(hasError: true),
      ],
      verify: (_) => verifyNever(repository.setWrprcStrictMode(enabled: anyNamed('enabled'))),
    );

    blocTest<DeveloperSettingsBloc, DeveloperSettingsState>(
      'clears a load error when retrying successfully',
      setUp: () => when(repository.getWrprcStrictMode()).thenAnswer((_) async => false),
      build: () => DeveloperSettingsBloc(repository),
      seed: () => const DeveloperSettingsState(hasError: true),
      act: (bloc) => bloc.add(const DeveloperSettingsLoadTriggered()),
      expect: () => [
        const DeveloperSettingsState(isBusy: true),
        const DeveloperSettingsState(wrprcStrictMode: false),
      ],
    );
  });

  group('toggle', () {
    for (final enabled in [true, false]) {
      blocTest<DeveloperSettingsBloc, DeveloperSettingsState>(
        'confirms strict mode $enabled after saving and clears any previous error',
        build: () => DeveloperSettingsBloc(repository),
        seed: () => DeveloperSettingsState(wrprcStrictMode: !enabled, hasError: true),
        act: (bloc) => bloc.add(DeveloperSettingsWrprcStrictModeToggled(enabled: enabled)),
        expect: () => [
          DeveloperSettingsState(wrprcStrictMode: !enabled, isBusy: true),
          DeveloperSettingsState(wrprcStrictMode: enabled),
        ],
        verify: (_) => verify(repository.setWrprcStrictMode(enabled: enabled)).called(1),
      );

      blocTest<DeveloperSettingsBloc, DeveloperSettingsState>(
        'preserves the confirmed value when saving strict mode $enabled fails',
        setUp: () => when(repository.setWrprcStrictMode(enabled: enabled)).thenAnswer(
          (_) async => throw StateError('save failed'),
        ),
        build: () => DeveloperSettingsBloc(repository),
        seed: () => DeveloperSettingsState(wrprcStrictMode: !enabled),
        act: (bloc) => bloc.add(DeveloperSettingsWrprcStrictModeToggled(enabled: enabled)),
        expect: () => [
          DeveloperSettingsState(wrprcStrictMode: !enabled, isBusy: true),
          DeveloperSettingsState(wrprcStrictMode: !enabled, hasError: true),
        ],
      );
    }

    blocTest<DeveloperSettingsBloc, DeveloperSettingsState>(
      'does not save before a value has been loaded',
      build: () => DeveloperSettingsBloc(repository),
      act: (bloc) => bloc.add(const DeveloperSettingsWrprcStrictModeToggled(enabled: false)),
      expect: () => [],
      verify: (_) => verifyNever(repository.setWrprcStrictMode(enabled: anyNamed('enabled'))),
    );
  });

  group('overlapping requests', () {
    late Completer<bool> load;
    late Completer<void> save;

    setUp(() {
      load = Completer<bool>();
      save = Completer<void>();
    });

    blocTest<DeveloperSettingsBloc, DeveloperSettingsState>(
      'ignores additional load and toggle requests while loading',
      setUp: () => when(repository.getWrprcStrictMode()).thenAnswer((_) => load.future),
      build: () => DeveloperSettingsBloc(repository),
      act: (bloc) async {
        bloc.add(const DeveloperSettingsLoadTriggered());
        await bloc.stream.firstWhere((state) => state.isBusy);
        bloc
          ..add(const DeveloperSettingsLoadTriggered())
          ..add(const DeveloperSettingsWrprcStrictModeToggled(enabled: false));
        await Future<void>.delayed(Duration.zero);
        load.complete(true);
        await bloc.stream.firstWhere((state) => !state.isBusy);
      },
      expect: () => [
        const DeveloperSettingsState(isBusy: true),
        const DeveloperSettingsState(wrprcStrictMode: true),
      ],
      verify: (_) {
        verify(repository.getWrprcStrictMode()).called(1);
        verifyNever(repository.setWrprcStrictMode(enabled: anyNamed('enabled')));
      },
    );

    blocTest<DeveloperSettingsBloc, DeveloperSettingsState>(
      'retains the confirmed value and ignores additional requests until saving completes',
      setUp: () => when(repository.setWrprcStrictMode(enabled: false)).thenAnswer((_) => save.future),
      build: () => DeveloperSettingsBloc(repository),
      seed: () => const DeveloperSettingsState(wrprcStrictMode: true),
      act: (bloc) async {
        bloc.add(const DeveloperSettingsWrprcStrictModeToggled(enabled: false));
        await bloc.stream.firstWhere((state) => state.isBusy);
        bloc
          ..add(const DeveloperSettingsLoadTriggered())
          ..add(const DeveloperSettingsWrprcStrictModeToggled(enabled: true));
        await Future<void>.delayed(Duration.zero);
        expect(bloc.state.wrprcStrictMode, isTrue);
        save.complete();
        await bloc.stream.firstWhere((state) => !state.isBusy);
      },
      expect: () => [
        const DeveloperSettingsState(wrprcStrictMode: true, isBusy: true),
        const DeveloperSettingsState(wrprcStrictMode: false),
      ],
      verify: (_) {
        verify(repository.setWrprcStrictMode(enabled: false)).called(1);
        verifyNever(repository.setWrprcStrictMode(enabled: true));
        verifyNever(repository.getWrprcStrictMode());
      },
    );
  });
}
