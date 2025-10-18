import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/event/listener/wallet_transfer_event_listener.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/domain/model/wallet_state.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockNavigationService navigationService;
  late MockGetWalletStateUseCase getWalletStateUseCase;
  late MockCancelWalletTransferUseCase cancelWalletTransferUseCase;
  late WalletTransferEventListener listener;

  setUp(() {
    provideDummy<WalletState>(WalletStateReady());

    navigationService = MockNavigationService();
    getWalletStateUseCase = MockGetWalletStateUseCase();
    cancelWalletTransferUseCase = MockCancelWalletTransferUseCase();
    listener = WalletTransferEventListener(
      navigationService,
      getWalletStateUseCase,
      cancelWalletTransferUseCase,
    );
  });

  group('onWalletUnlocked', () {
    test(
      'should navigate to wallet transfer screen when state is transfer possible (with isRetry=false)',
      () async {
        when(getWalletStateUseCase.invoke()).thenAnswer(
          (_) async => WalletStateTransferPossible(),
        );

        await listener.onWalletUnlocked();

        verify(
          navigationService.handleNavigationRequest(
            NavigationRequest.walletTransferTarget(isRetry: false),
            queueIfNotReady: true,
          ),
        ).called(1);
        verifyNever(cancelWalletTransferUseCase.invoke());
      },
    );

    test(
      'should cancel transfer and navigate to transfer screen when state is transferring as target (with isRetry=true)',
      () async {
        when(getWalletStateUseCase.invoke()).thenAnswer(
          (_) async => WalletStateTransferring(TransferRole.target),
        );

        await listener.onWalletUnlocked();

        verify(cancelWalletTransferUseCase.invoke()).called(1);
        verify(
          navigationService.handleNavigationRequest(
            NavigationRequest.walletTransferTarget(isRetry: true),
            queueIfNotReady: true,
          ),
        ).called(1);
      },
    );

    test(
      'should cancel transfer and not navigate when state is transferring as source',
      () async {
        when(getWalletStateUseCase.invoke()).thenAnswer(
          (_) async => WalletStateTransferring(TransferRole.source),
        );

        await listener.onWalletUnlocked();

        verify(cancelWalletTransferUseCase.invoke()).called(1);
        verifyNever(navigationService.handleNavigationRequest(any, queueIfNotReady: anyNamed('queueIfNotReady')));
      },
    );

    test(
      'should do nothing when wallet state is not transfer related',
      () async {
        when(getWalletStateUseCase.invoke()).thenAnswer((_) async => WalletStateReady());

        await listener.onWalletUnlocked();

        verifyNever(cancelWalletTransferUseCase.invoke());
        verifyNever(navigationService.handleNavigationRequest(any, queueIfNotReady: anyNamed('queueIfNotReady')));
      },
    );

    test(
      'should do nothing when irrelevant listeners are triggered',
      () async {
        await listener.onDashboardShown();
        await listener.onWalletLocked();

        verifyNever(cancelWalletTransferUseCase.invoke());
        verifyNever(navigationService.handleNavigationRequest(any, queueIfNotReady: anyNamed('queueIfNotReady')));
      },
    );
  });
}
