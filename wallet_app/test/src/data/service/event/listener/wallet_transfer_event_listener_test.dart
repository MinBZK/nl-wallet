import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/event/listener/wallet_transfer_event_listener.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/domain/model/wallet_status.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockNavigationService navigationService;
  late MockGetWalletStatusUseCase getWalletStatusUseCase;
  late MockCancelWalletTransferUseCase cancelWalletTransferUseCase;
  late WalletTransferEventListener listener;

  setUp(() {
    provideDummy<WalletStatus>(WalletStatusReady());

    navigationService = MockNavigationService();
    getWalletStatusUseCase = MockGetWalletStatusUseCase();
    cancelWalletTransferUseCase = MockCancelWalletTransferUseCase();
    listener = WalletTransferEventListener(
      navigationService,
      getWalletStatusUseCase,
      cancelWalletTransferUseCase,
    );
  });

  group('onWalletUnlocked', () {
    test(
      'should navigate to wallet transfer screen when status is transfer possible (with isRetry=false)',
      () async {
        when(getWalletStatusUseCase.invoke()).thenAnswer(
          (_) async => WalletStatusTransferPossible(),
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
      'should cancel transfer and navigate to transfer screen when status is transferring as target (with isRetry=true)',
      () async {
        when(getWalletStatusUseCase.invoke()).thenAnswer(
          (_) async => WalletStatusTransferring(TransferRole.target),
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
      'should cancel transfer and not navigate when status is transferring as source',
      () async {
        when(getWalletStatusUseCase.invoke()).thenAnswer(
          (_) async => WalletStatusTransferring(TransferRole.source),
        );

        await listener.onWalletUnlocked();

        verify(cancelWalletTransferUseCase.invoke()).called(1);
        verifyNever(navigationService.handleNavigationRequest(any, queueIfNotReady: anyNamed('queueIfNotReady')));
      },
    );

    test(
      'should do nothing when wallet status is not transfer related',
      () async {
        when(getWalletStatusUseCase.invoke()).thenAnswer((_) async => WalletStatusReady());

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
