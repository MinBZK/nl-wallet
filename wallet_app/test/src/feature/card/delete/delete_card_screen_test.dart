import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/data/repository/card/wallet_card_repository.dart';
import 'package:wallet/src/feature/card/delete/bloc/delete_card_bloc.dart';
import 'package:wallet/src/feature/card/delete/delete_card_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.mocks.dart';
import '../../../test_util/golden_utils.dart';

class MockDeleteCardBloc extends MockBloc<DeleteCardEvent, DeleteCardState> implements DeleteCardBloc {}

void main() {
  group('DeleteCardScreen', () {
    late MockDeleteCardBloc mockBloc;

    setUp(() {
      mockBloc = MockDeleteCardBloc();
    });

    group('goldens', () {
      testGoldens('DeleteCardProvidePin', (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const DeleteCardScreen().withState<DeleteCardBloc, DeleteCardState>(
            mockBloc,
            const DeleteCardProvidePin(attestationId: 'card-123', cardTitle: 'Driving License'),
          ),
          providers: [
            RepositoryProvider<WalletCardRepository>(create: (c) => MockWalletCardRepository()),
          ],
        );
        await screenMatchesGolden('delete_card_provide_pin');
      });

      testGoldens('DeleteCardSuccess', (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const DeleteCardScreen().withState<DeleteCardBloc, DeleteCardState>(
            mockBloc,
            const DeleteCardSuccess(cardTitle: 'Driving License'),
          ),
        );
        await screenMatchesGolden('delete_card_success');
      });

      testGoldens('DeleteCardSuccess - dark', (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const DeleteCardScreen().withState<DeleteCardBloc, DeleteCardState>(
            mockBloc,
            const DeleteCardSuccess(cardTitle: 'Driving License'),
          ),
          brightness: Brightness.dark,
        );
        await screenMatchesGolden('delete_card_success.dark');
      });

      testGoldens('DeleteCardSuccess - landscape', (tester) async {
        await tester.pumpWidgetWithAppWrapper(
          const DeleteCardScreen().withState<DeleteCardBloc, DeleteCardState>(
            mockBloc,
            const DeleteCardSuccess(cardTitle: 'Driving License'),
          ),
          surfaceSize: iphoneXSizeLandscape,
        );
        await screenMatchesGolden('delete_card_success.landscape');
      });
    });
  });
}
