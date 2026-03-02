import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/wallet_state.dart';
import 'package:wallet/src/feature/blocked/bloc/app_blocked_bloc.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockGetWalletStateUseCase mockGetWalletStateUseCase;

  setUp(() {
    mockGetWalletStateUseCase = MockGetWalletStateUseCase();
  });

  group('AppBlockedBloc', () {
    blocTest<AppBlockedBloc, AppBlockedState>(
      'emits [AppBlockedInitial, AppBlockedByUser] when reason is userRequest',
      build: () => AppBlockedBloc(mockGetWalletStateUseCase),
      act: (bloc) => bloc.add(const AppBlockedLoadTriggered(reason: RevocationReason.userRequest)),
      wait: const Duration(milliseconds: 500),
      expect: () => [
        AppBlockedInitial(),
        const AppBlockedByUser(),
      ],
    );

    blocTest<AppBlockedBloc, AppBlockedState>(
      'emits [AppBlockedInitial, AppBlockedByAdmin] when reason is adminRequest',
      setUp: () {
        when(mockGetWalletStateUseCase.invoke()).thenAnswer(
          (_) async => const WalletStateBlocked(BlockedReason.blockedByWalletProvider, canRegisterNewAccount: true),
        );
      },
      build: () => AppBlockedBloc(mockGetWalletStateUseCase),
      act: (bloc) => bloc.add(const AppBlockedLoadTriggered(reason: RevocationReason.adminRequest)),
      wait: const Duration(milliseconds: 500),
      expect: () => [
        AppBlockedInitial(),
        const AppBlockedByAdmin(
          WalletStateBlocked(BlockedReason.blockedByWalletProvider, canRegisterNewAccount: true),
        ),
      ],
      verify: (_) => verify(mockGetWalletStateUseCase.invoke()).called(1),
    );

    blocTest<AppBlockedBloc, AppBlockedState>(
      'emits [AppBlockedInitial, AppBlockedByAdmin] when reason is unknown',
      setUp: () {
        when(mockGetWalletStateUseCase.invoke()).thenAnswer(
          (_) async => const WalletStateBlocked(BlockedReason.blockedByWalletProvider, canRegisterNewAccount: true),
        );
      },
      build: () => AppBlockedBloc(mockGetWalletStateUseCase),
      act: (bloc) => bloc.add(const AppBlockedLoadTriggered(reason: RevocationReason.unknown)),
      wait: const Duration(milliseconds: 500),
      expect: () => [
        AppBlockedInitial(),
        const AppBlockedByAdmin(
          WalletStateBlocked(BlockedReason.blockedByWalletProvider, canRegisterNewAccount: true),
        ),
      ],
      verify: (_) => verify(mockGetWalletStateUseCase.invoke()).called(1),
    );

    blocTest<AppBlockedBloc, AppBlockedState>(
      'emits [AppBlockedInitial, AppBlockedError] when getWalletStateUseCase throws',
      setUp: () {
        when(mockGetWalletStateUseCase.invoke()).thenThrow(Exception('Wallet not blocked'));
      },
      build: () => AppBlockedBloc(mockGetWalletStateUseCase),
      act: (bloc) => bloc.add(const AppBlockedLoadTriggered(reason: RevocationReason.adminRequest)),
      wait: const Duration(milliseconds: 500),
      expect: () => [
        AppBlockedInitial(),
        const AppBlockedError(),
      ],
    );
  });
}
