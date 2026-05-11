import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/history/detail/bloc/history_detail_bloc.dart';

import '../../../../mocks/wallet_mock_data.dart';

void main() {
  group('HistoryDetailState', () {
    group('HistoryDetailInitial', () {
      test('supports value equality', () {
        expect(
          const HistoryDetailInitial(),
          const HistoryDetailInitial(),
        );
      });
    });

    group('HistoryDetailLoadInProgress', () {
      test('supports value equality', () {
        expect(
          const HistoryDetailLoadInProgress(),
          const HistoryDetailLoadInProgress(),
        );
      });
    });

    group('HistoryDetailLoadSuccess', () {
      test('supports value equality', () {
        expect(
          HistoryDetailLoadSuccess(WalletMockData.disclosureEvent),
          HistoryDetailLoadSuccess(WalletMockData.disclosureEvent),
        );
      });

      test('supperts value in-equality', () {
        expect(
          HistoryDetailLoadSuccess(WalletMockData.disclosureEvent),
          isNot(HistoryDetailLoadSuccess(WalletMockData.deletionEvent)),
        );
      });
    });

    group('HistoryDetailLoadFailure', () {
      test('supports value equality', () {
        expect(
          const HistoryDetailLoadFailure(GenericError('test', sourceError: '')),
          const HistoryDetailLoadFailure(GenericError('test', sourceError: '')),
        );
      });
      test('supports value in-equality', () {
        expect(
          const HistoryDetailLoadFailure(GenericError('test', sourceError: '')),
          isNot(const HistoryDetailLoadFailure(GenericError('alt', sourceError: ''))),
        );
      });
    });
  });
}
