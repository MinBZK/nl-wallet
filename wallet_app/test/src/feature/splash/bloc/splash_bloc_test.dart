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
    'if mockGetWalletStateUseCase throws, app initialization should fail',
    setUp: () => when(mockGetWalletStateUseCase.invoke()).thenThrow(StateError('error')),
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    errors: () => [isA<StateError>()],
  );

  blocTest(
    'Verify user redirected to onboarding when wallet state is WalletStateRegistration',
    setUp: () => when(
      mockGetWalletStateUseCase.invoke(),
    ).thenAnswer((_) async => const WalletStateUnregistered()),
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.onboarding)],
  );

  blocTest(
    'verify user is redirected to pidRetrieval when wallet state is WalletStateEmpty',
    setUp: () => when(
      mockGetWalletStateUseCase.invoke(),
    ).thenAnswer((_) async => const WalletStateEmpty()),
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.pidRetrieval)],
  );

  blocTest(
    'verify user is also redirected to pidRetrieval when wallet state is WalletStateLocked with substate of WalletStateEmpty',
    setUp: () => when(
      mockGetWalletStateUseCase.invoke(),
    ).thenAnswer((_) async => const WalletStateLocked(WalletStateEmpty())),
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.pidRetrieval)],
  );

  blocTest(
    'verify user is redirected to dashboard when wallet state is WalletStateReady',
    setUp: () => when(
      mockGetWalletStateUseCase.invoke(),
    ).thenAnswer((_) async => const WalletStateReady()),
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.dashboard)],
  );

  blocTest(
    'verify user is also redirected to dashboard when wallet state is WalletStateLocked with substate of WalletStateReady',
    setUp: () => when(
      mockGetWalletStateUseCase.invoke(),
    ).thenAnswer((_) async => const WalletStateLocked(WalletStateReady())),
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.dashboard)],
  );

  blocTest(
    'verify user is redirected to transfer when wallet state is WalletStateTransferPossible',
    setUp: () => when(
      mockGetWalletStateUseCase.invoke(),
    ).thenAnswer((_) async => const WalletStateTransferPossible()),
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.transfer)],
  );

  blocTest(
    'verify user is redirected to transfer when wallet state is WalletStateTransferring with target role',
    setUp: () => when(mockGetWalletStateUseCase.invoke()).thenAnswer(
      (_) async => const WalletStateTransferring(TransferRole.target),
    ),
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.transfer)],
  );

  blocTest(
    'verify user is redirected to dashboard when wallet state is WalletStateTransferring with source role',
    setUp: () => when(mockGetWalletStateUseCase.invoke()).thenAnswer(
      (_) async => const WalletStateTransferring(TransferRole.source),
    ),
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.dashboard)],
  );

  blocTest(
    'verify user is redirected to blocked when wallet state is WalletStateWalletBlocked',
    setUp: () => when(mockGetWalletStateUseCase.invoke()).thenAnswer(
      (_) async => const WalletStateBlocked(BlockedReason.requiresAppUpdate),
    ),
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.blocked)],
  );

  blocTest(
    'verify user is redirected to dashboard when wallet state is WalletStateDisclosure',
    setUp: () => when(
      mockGetWalletStateUseCase.invoke(),
    ).thenAnswer((_) async => const WalletStateInDisclosureFlow()),
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.dashboard)],
  );

  blocTest(
    'verify user is redirected to dashboard when wallet state is WalletStateIssuance',
    setUp: () => when(
      mockGetWalletStateUseCase.invoke(),
    ).thenAnswer((_) async => const WalletStateInIssuanceFlow()),
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.dashboard)],
  );

  blocTest(
    'verify user is redirected to dashboard when wallet state is WalletStatePinChange',
    setUp: () => when(
      mockGetWalletStateUseCase.invoke(),
    ).thenAnswer((_) async => const WalletStateInPinChangeFlow()),
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.dashboard)],
  );

  blocTest(
    'verify user is redirected to pinRecovery when wallet state is WalletStatePinRecovery',
    setUp: () => when(
      mockGetWalletStateUseCase.invoke(),
    ).thenAnswer((_) async => const WalletStateInPinRecoveryFlow()),
    act: (bloc) => bloc.add(const InitSplashEvent()),
    build: () => SplashBloc(mockGetWalletStateUseCase),
    expect: () => [const SplashLoaded(.pinRecovery)],
  );
}
