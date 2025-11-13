import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/transfer/cancel_wallet_transfer_usecase.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/wallet_transfer_source/page/wallet_transfer_source_confirm_pin_page.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.mocks.dart';

class MockPinBloc extends MockBloc<PinEvent, PinState> implements PinBloc {}

void main() {
  late MockCancelWalletTransferUseCase mockCancelWalletTransferUseCase;
  late MockPinBloc pinBloc;

  setUp(() {
    mockCancelWalletTransferUseCase = MockCancelWalletTransferUseCase();
    pinBloc = MockPinBloc();
  });

  Widget createTestWidget() {
    return MultiRepositoryProvider(
      providers: [
        RepositoryProvider<CancelWalletTransferUseCase>.value(value: mockCancelWalletTransferUseCase),
        BlocProvider<PinBloc>.value(value: pinBloc),
      ],
      child: WalletTransferSourceConfirmPinPage(
        onPinConfirmed: (_) {},
        onPinConfirmationFailed: (context, state) {},
      ),
    );
  }

  group('WalletTransferSourceConfirmPinPage', () {
    testWidgets(
      'invokes CancelWalletTransferUseCase when PinBloc emits PinValidateTimeout',
      (tester) async {
        whenListen(
          pinBloc,
          Stream.fromIterable([PinValidateTimeout(DateTime(1337))]),
          initialState: const PinEntryInProgress(0),
        );

        await tester.pumpWidgetWithAppWrapper(createTestWidget());

        verify(mockCancelWalletTransferUseCase.invoke()).called(1);
      },
    );

    testWidgets(
      'invokes CancelWalletTransferUseCase when PinBloc emits PinValidateBlocked',
      (tester) async {
        whenListen(
          pinBloc,
          Stream.fromIterable(const [PinValidateBlocked()]),
          initialState: const PinEntryInProgress(0),
        );

        await tester.pumpWidgetWithAppWrapper(createTestWidget());

        verify(mockCancelWalletTransferUseCase.invoke()).called(1);
      },
    );
  });
}
