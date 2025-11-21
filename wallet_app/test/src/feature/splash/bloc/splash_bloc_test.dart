import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/wallet_state.dart';
import 'package:wallet/src/feature/splash/bloc/splash_bloc.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockGetWalletStateUseCase mockGetWalletStateUseCase;

  setUp(() {
    mockGetWalletStateUseCase = MockGetWalletStateUseCase();
  });

  blocTest(
    'verify initial state',
    build: () => SplashBloc(mockGetWalletStateUseCase),
    verify: (bloc) => expect(bloc.state, SplashInitial()),
  );

  blocTest(
    'if mockGetWalletStateUseCase app initialization should fail',
    setUp: () {
      when(mockGetWalletStateUseCase.invoke()).thenThrow(StateError('error'));
    },
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    errors: () => [isA<StateError>()],
  );

  blocTest(
    'if mockGetWalletStateUseCase app initialization should fail',
    setUp: () {
      when(mockGetWalletStateUseCase.invoke()).thenThrow(StateError('error'));
    },
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    errors: () => [isA<StateError>()],
  );

  blocTest(
    'validate state when locked, registered and pid not available',
    setUp: () {
      when(mockGetWalletStateUseCase.invoke()).thenAnswer((_) async => const WalletStateLocked(WalletStateEmpty()));
    },
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.pidRetrieval)],
  );

  blocTest(
    'validate state when registered and pid not available',
    setUp: () {
      when(mockGetWalletStateUseCase.invoke()).thenAnswer((_) async => const WalletStateEmpty());
    },
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.pidRetrieval)],
  );

  blocTest(
    'validate state when not registered',
    setUp: () {
      when(mockGetWalletStateUseCase.invoke()).thenAnswer((_) async => WalletStateRegistration());
    },
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.onboarding)],
  );

  blocTest(
    'validate state when registered and pid available',
    setUp: () {
      when(mockGetWalletStateUseCase.invoke()).thenAnswer((_) async => const WalletStateReady());
    },
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.dashboard)],
  );

  blocTest(
    'validate state when locked, registered and pid available',
    setUp: () {
      when(mockGetWalletStateUseCase.invoke()).thenAnswer((_) async => const WalletStateLocked(WalletStateReady()));
    },
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.dashboard)],
  );
}
