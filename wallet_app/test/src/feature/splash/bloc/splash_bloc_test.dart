import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/feature/splash/bloc/splash_bloc.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockIsWalletInitializedUseCase isWalletInitializedUseCase;
  late MockIsWalletInitializedWithPidUseCase isWalletInitializedWithPidUseCase;

  setUp(() {
    isWalletInitializedUseCase = MockIsWalletInitializedUseCase();
    isWalletInitializedWithPidUseCase = MockIsWalletInitializedWithPidUseCase();
  });

  blocTest(
    'verify initial state',
    build: () => SplashBloc(isWalletInitializedUseCase, isWalletInitializedWithPidUseCase),
    verify: (bloc) => expect(bloc.state, SplashInitial()),
  );

  blocTest(
    'if isWalletInitializedUseCase app initialization should fail',
    setUp: () {
      when(isWalletInitializedUseCase.invoke()).thenThrow(StateError('error'));
    },
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(isWalletInitializedUseCase, isWalletInitializedWithPidUseCase),
    errors: () => [isA<StateError>()],
  );

  blocTest(
    'if isWalletInitializedWithPidUseCase app initialization should fail',
    setUp: () {
      when(isWalletInitializedUseCase.invoke()).thenAnswer((_) async => true);
      when(isWalletInitializedWithPidUseCase.invoke()).thenThrow(StateError('error'));
    },
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(isWalletInitializedUseCase, isWalletInitializedWithPidUseCase),
    errors: () => [isA<StateError>()],
  );

  blocTest(
    'validate state when not initialized and pid not available',
    setUp: () {
      when(isWalletInitializedUseCase.invoke()).thenAnswer((_) async => false);
      when(isWalletInitializedWithPidUseCase.invoke()).thenAnswer((_) async => false);
    },
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(isWalletInitializedUseCase, isWalletInitializedWithPidUseCase),
    expect: () => [const SplashLoaded(isRegistered: false, hasPid: false)],
  );

  blocTest(
    'validate state when initialized and pid not available',
    setUp: () {
      when(isWalletInitializedUseCase.invoke()).thenAnswer((_) async => true);
      when(isWalletInitializedWithPidUseCase.invoke()).thenAnswer((_) async => false);
    },
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(isWalletInitializedUseCase, isWalletInitializedWithPidUseCase),
    expect: () => [const SplashLoaded(isRegistered: true, hasPid: false)],
  );

  blocTest(
    'validate state when initialized and pid available',
    setUp: () {
      when(isWalletInitializedUseCase.invoke()).thenAnswer((_) async => true);
      when(isWalletInitializedWithPidUseCase.invoke()).thenAnswer((_) async => true);
    },
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(isWalletInitializedUseCase, isWalletInitializedWithPidUseCase),
    expect: () => [const SplashLoaded(isRegistered: true, hasPid: true)],
  );
}
