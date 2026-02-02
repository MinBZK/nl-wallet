import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/usecase/revocation/get_revocation_code_usecase.dart';
import 'package:wallet/src/feature/review_revocation_code_screen/bloc/review_revocation_code_bloc.dart';
import 'package:wallet/src/feature/review_revocation_code_screen/review_revocation_code_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.mocks.dart';
import '../../test_util/golden_utils.dart';

class MockReviewRevocationCodeBloc extends MockBloc<ReviewRevocationCodeEvent, ReviewRevocationCodeState>
    implements ReviewRevocationCodeBloc {}

const sampleRevocationCode = 'AB12CD34EF56GH78IJ';

void main() {
  group('ReviewRevocationCodeScreen', () {
    late MockReviewRevocationCodeBloc mockBloc;
    late MockGetRevocationCodeUseCase mockGetRevocationCodeUseCase;

    setUp(() {
      mockBloc = MockReviewRevocationCodeBloc();
      mockGetRevocationCodeUseCase = MockGetRevocationCodeUseCase();
    });

    testGoldens('ReviewRevocationCodeInitial', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ReviewRevocationCodeScreen().withState<ReviewRevocationCodeBloc, ReviewRevocationCodeState>(
          mockBloc,
          const ReviewRevocationCodeInitial(),
        ),
      );
      await screenMatchesGolden('review_revocation_code_initial');
    });

    testGoldens('ReviewRevocationCodeProvidePin', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        RepositoryProvider<GetRevocationCodeUseCase>.value(
          value: mockGetRevocationCodeUseCase,
          child: const ReviewRevocationCodeScreen().withState<ReviewRevocationCodeBloc, ReviewRevocationCodeState>(
            mockBloc,
            const ReviewRevocationCodeProvidePin(),
          ),
        ),
      );
      await screenMatchesGolden('review_revocation_code_provide_pin');
    });

    testGoldens('ReviewRevocationCodeSuccess', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ReviewRevocationCodeScreen().withState<ReviewRevocationCodeBloc, ReviewRevocationCodeState>(
          mockBloc,
          const ReviewRevocationCodeSuccess(sampleRevocationCode),
        ),
      );
      await screenMatchesGolden('review_revocation_code_success');
    });

    testGoldens('ReviewRevocationCodeSuccess - dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ReviewRevocationCodeScreen().withState<ReviewRevocationCodeBloc, ReviewRevocationCodeState>(
          mockBloc,
          const ReviewRevocationCodeSuccess(sampleRevocationCode),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('review_revocation_code_success.dark');
    });
  });
}
