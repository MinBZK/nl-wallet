import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/feature/qr/present/bloc/qr_present_bloc.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockStartQrEngagementUseCase mockStartQrEngagementUseCase;
  late QrPresentBloc bloc;

  setUp(() {
    mockStartQrEngagementUseCase = MockStartQrEngagementUseCase();
    bloc = QrPresentBloc(mockStartQrEngagementUseCase);
  });

  group('QrPresentBloc', () {
    test('initial state is QrPresentInitial', () {
      expect(bloc.state, const QrPresentInitial());
    });

    blocTest<QrPresentBloc, QrPresentState>(
      'emits [QrPresentInitial, QrPresentServerStarted] when QrPresentStartRequested is added and usecase succeeds',
      build: () => bloc,
      setUp: () {
        when(mockStartQrEngagementUseCase.invoke()).thenAnswer((_) async => const Result.success('qr_content'));
      },
      act: (bloc) => bloc.add(const QrPresentStartRequested()),
      expect: () => [
        const QrPresentInitial(),
        const QrPresentServerStarted('qr_content'),
      ],
      verify: (_) {
        verify(mockStartQrEngagementUseCase.invoke()).called(1);
      },
    );

    blocTest<QrPresentBloc, QrPresentState>(
      'emits [QrPresentInitial, QrPresentError] when QrPresentStartRequested is added and usecase fails',
      build: () => bloc,
      setUp: () {
        when(mockStartQrEngagementUseCase.invoke()).thenAnswer(
          (_) async => const Result.error(GenericError('error', sourceError: 'error')),
        );
      },
      act: (bloc) => bloc.add(const QrPresentStartRequested()),
      expect: () => [
        const QrPresentInitial(),
        const QrPresentError(GenericError('error', sourceError: 'error')),
      ],
      verify: (_) {
        verify(mockStartQrEngagementUseCase.invoke()).called(1);
      },
    );
  });
}
