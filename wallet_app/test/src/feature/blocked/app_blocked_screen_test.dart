import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/wallet_state.dart';
import 'package:wallet/src/domain/usecase/wallet/reset_wallet_usecase.dart';
import 'package:wallet/src/feature/blocked/app_blocked_screen.dart';
import 'package:wallet/src/feature/blocked/bloc/app_blocked_bloc.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.dart';
import '../../test_util/golden_utils.dart';

class MockAppBlockedBloc extends MockBloc<AppBlockedEvent, AppBlockedState> implements AppBlockedBloc {}

void main() {
  group('AppBlockedScreen Goldens', () {
    late AppBlockedBloc bloc;

    setUp(() {
      bloc = MockAppBlockedBloc();
    });

    testGoldens('AppBlockedInitial state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AppBlockedScreen().withState<AppBlockedBloc, AppBlockedState>(
          bloc,
          AppBlockedInitial(),
        ),
      );
      await screenMatchesGolden('app_blocked_initial');
    });

    testGoldens('AppBlockedError state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AppBlockedScreen().withState<AppBlockedBloc, AppBlockedState>(
          bloc,
          const AppBlockedError(),
        ),
      );
      await screenMatchesGolden('app_blocked_error');
    });

    testGoldens('AppBlockedByUser state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AppBlockedScreen().withState<AppBlockedBloc, AppBlockedState>(
          bloc,
          const AppBlockedByUser(),
        ),
      );
      await screenMatchesGolden('app_blocked_by_user');
    });

    testGoldens('AppBlockedByAdmin state - can register new account', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AppBlockedScreen().withState<AppBlockedBloc, AppBlockedState>(
          bloc,
          const AppBlockedByAdmin(
            WalletStateBlocked(
              BlockedReason.blockedByWalletProvider,
              canRegisterNewAccount: true,
            ),
          ),
        ),
        providers: [
          RepositoryProvider<ResetWalletUseCase>(create: (_) => MockResetWalletUseCase()),
        ],
      );
      await screenMatchesGolden('app_blocked_by_admin_can_register');
    });

    testGoldens('AppBlockedByAdmin state - cannot register new account', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AppBlockedScreen().withState<AppBlockedBloc, AppBlockedState>(
          bloc,
          const AppBlockedByAdmin(
            WalletStateBlocked(
              BlockedReason.blockedByWalletProvider,
              canRegisterNewAccount: false,
            ),
          ),
        ),
      );
      await screenMatchesGolden('app_blocked_by_admin_cannot_register');
    });

    testGoldens('AppBlockedByAdmin state - dark mode', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const AppBlockedScreen().withState<AppBlockedBloc, AppBlockedState>(
          bloc,
          const AppBlockedByAdmin(
            WalletStateBlocked(
              BlockedReason.blockedByWalletProvider,
              canRegisterNewAccount: true,
            ),
          ),
        ),
        brightness: Brightness.dark,
        providers: [
          RepositoryProvider<ResetWalletUseCase>(create: (_) => MockResetWalletUseCase()),
        ],
      );
      await screenMatchesGolden('app_blocked_by_admin_dark');
    });
  });
}
