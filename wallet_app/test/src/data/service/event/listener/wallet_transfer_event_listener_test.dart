import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/event/listener/wallet_transfer_app_event_listener.dart';
import 'package:wallet/src/domain/model/wallet_state.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockNavigationService navigationService;
  late MockGetWalletStateUseCase getWalletStateUseCase;
  late MockCancelWalletTransferUseCase cancelWalletTransferUseCase;
  late WalletTransferAppEventListener listener;

  setUp(() {
    navigationService = MockNavigationService();
    getWalletStateUseCase = MockGetWalletStateUseCase();
    cancelWalletTransferUseCase = MockCancelWalletTransferUseCase();
    listener = WalletTransferAppEventListener(
      navigationService,
      getWalletStateUseCase,
      cancelWalletTransferUseCase,
    );
  });

  group('onWalletUnlocked', () {
    test(
      'should cancel transfer and show moveStopped dialog',
      () async {
        when(getWalletStateUseCase.invoke()).thenAnswer(
          (_) async => const WalletStateTransferring(TransferRole.target),
        );

        await listener.onWalletUnlocked();

        verify(cancelWalletTransferUseCase.invoke()).called(1);
        verify(navigationService.showDialog(.moveStopped)).called(1);
      },
    );

    test(
      'should cancel transfer and not navigate when state is transferring as source',
      () async {
        when(getWalletStateUseCase.invoke()).thenAnswer(
          (_) async => const WalletStateTransferring(TransferRole.source),
        );

        await listener.onWalletUnlocked();

        verify(cancelWalletTransferUseCase.invoke()).called(1);
        verifyNever(navigationService.handleNavigationRequest(any, queueIfNotReady: anyNamed('queueIfNotReady')));
      },
    );

    test(
      'should do nothing when wallet state is not transfer related',
      () async {
        when(getWalletStateUseCase.invoke()).thenAnswer((_) async => const WalletStateReady());

        await listener.onWalletUnlocked();

        verifyNever(cancelWalletTransferUseCase.invoke());
        verifyNever(navigationService.handleNavigationRequest(any, queueIfNotReady: anyNamed('queueIfNotReady')));
      },
    );

    test(
      'should cancel wallet transfer onDashboardShown on source device',
      () async {
        when(
          getWalletStateUseCase.invoke(),
        ).thenAnswer((_) async => const WalletStateTransferring(TransferRole.source));
        await listener.onDashboardShown();

        verify(cancelWalletTransferUseCase.invoke()).called(1);
      },
    );

    test(
      'should NOT cancel wallet transfer onDashboardShown on destination device',
      () async {
        when(
          getWalletStateUseCase.invoke(),
        ).thenAnswer((_) async => const WalletStateTransferring(TransferRole.target));
        await listener.onDashboardShown();

        verifyNever(cancelWalletTransferUseCase.invoke());
      },
    );

    test(
      'should do nothing when irrelevant listeners are triggered',
      () async {
        await listener.onWalletLocked();

        verifyNever(cancelWalletTransferUseCase.invoke());
        verifyNever(navigationService.handleNavigationRequest(any, queueIfNotReady: anyNamed('queueIfNotReady')));
      },
    );
  });
}
