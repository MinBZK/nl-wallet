import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/home/bloc/home_bloc.dart';

void main() {
  late HomeBloc bloc;

  setUp(() {
    bloc = HomeBloc();
  });

  group('initialState', () {
    test('state should have `cards` tab selected when initialised', () {
      expect(
          bloc.state,
          const HomeScreenSelect(
            HomeTab.cards,
          ));
    });
  });

  group('eventHomeTabPressed', () {
    blocTest<HomeBloc, HomeState>(
      'state should have `cards` tab selected when tab `cards` is pressed',
      build: () => bloc,
      act: (bloc) => bloc.add(const HomeTabPressed(HomeTab.cards)),
      expect: () => [const HomeScreenSelect(HomeTab.cards)],
    );

    blocTest<HomeBloc, HomeState>(
      'state should have `qr` tab selected when tab `qr` is pressed',
      build: () => bloc,
      act: (bloc) => bloc.add(const HomeTabPressed(HomeTab.qr)),
      expect: () => [const HomeScreenSelect(HomeTab.qr)],
    );

    blocTest<HomeBloc, HomeState>(
      'state should have `menu` tab selected when tab `menu` is pressed',
      build: () => bloc,
      act: (bloc) => bloc.add(const HomeTabPressed(HomeTab.menu)),
      expect: () => [const HomeScreenSelect(HomeTab.menu)],
    );
  });

  group('forceStateRefresh', () {
    blocTest<HomeBloc, HomeState>(
      'should not set `uid` when `forceStateRefresh = false`',
      build: () => bloc,
      act: (bloc) => bloc.add(const HomeTabPressed(HomeTab.menu, forceStateRefresh: false)),
      expect: () => [const HomeScreenSelect(HomeTab.menu, uid: null)],
    );

    blocTest<HomeBloc, HomeState>('should set `uid` when `forceStateRefresh = true`',
        build: () => bloc,
        act: (bloc) => bloc.add(const HomeTabPressed(HomeTab.menu, forceStateRefresh: true)),
        verify: (bloc) => expect((bloc.state as HomeScreenSelect).uid, isNotNull));
  });
}
