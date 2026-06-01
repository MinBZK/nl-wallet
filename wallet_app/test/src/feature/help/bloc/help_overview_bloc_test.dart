import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/help/help_category.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/feature/help/bloc/help_overview_bloc.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  const locale = Locale('en');

  late MockGetHelpCategoriesUseCase getHelpCategoriesUseCase;

  setUp(() {
    getHelpCategoriesUseCase = MockGetHelpCategoriesUseCase();
    when(getHelpCategoriesUseCase.invoke(any)).thenAnswer((_) async => const Result.success(<HelpCategory>[]));
  });

  HelpOverviewBloc buildBloc() => HelpOverviewBloc(getHelpCategoriesUseCase, locale);

  blocTest<HelpOverviewBloc, HelpOverviewState>(
    'initial state is HelpOverviewInitial',
    build: buildBloc,
    verify: (bloc) => expect(bloc.state, isA<HelpOverviewInitial>()),
  );

  blocTest<HelpOverviewBloc, HelpOverviewState>(
    'emits [InProgress, LoadSuccess] when the usecase returns categories',
    build: () {
      when(getHelpCategoriesUseCase.invoke(any)).thenAnswer(
        (_) async => const Result.success([
          HelpCategory(id: 'cat1', icon: 'play_arrow', title: 'Category 1', subtitle: 'Subtitle', subcategories: []),
        ]),
      );
      return buildBloc();
    },
    act: (bloc) => bloc.add(const HelpOverviewLoadTriggered()),
    expect: () => const <HelpOverviewState>[
      HelpOverviewLoadInProgress(),
      HelpOverviewLoadSuccess([
        HelpCategory(id: 'cat1', icon: 'play_arrow', title: 'Category 1', subtitle: 'Subtitle', subcategories: []),
      ]),
    ],
  );

  blocTest<HelpOverviewBloc, HelpOverviewState>(
    'emits [InProgress, LoadSuccess] with an empty list when the usecase returns no categories',
    build: buildBloc,
    act: (bloc) => bloc.add(const HelpOverviewLoadTriggered()),
    expect: () => const <HelpOverviewState>[
      HelpOverviewLoadInProgress(),
      HelpOverviewLoadSuccess([]),
    ],
  );

  blocTest<HelpOverviewBloc, HelpOverviewState>(
    'emits [InProgress, LoadFailure] when the usecase fails',
    build: () {
      when(getHelpCategoriesUseCase.invoke(any)).thenAnswer(
        (_) async => const Result.error(GenericError('unsupported locale', sourceError: 'locale')),
      );
      return buildBloc();
    },
    act: (bloc) => bloc.add(const HelpOverviewLoadTriggered()),
    expect: () => const <HelpOverviewState>[
      HelpOverviewLoadInProgress(),
      HelpOverviewLoadFailure(GenericError('unsupported locale', sourceError: 'locale')),
    ],
  );

  group('events', () {
    test('HelpOverviewLoadTriggered instances are equal', () {
      expect(const HelpOverviewLoadTriggered(), equals(const HelpOverviewLoadTriggered()));
    });
  });
}
