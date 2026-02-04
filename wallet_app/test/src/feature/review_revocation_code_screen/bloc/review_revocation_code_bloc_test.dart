import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/review_revocation_code_screen/bloc/review_revocation_code_bloc.dart';

void main() {
  group('ReviewRevocationCodeBloc', () {
    blocTest<ReviewRevocationCodeBloc, ReviewRevocationCodeState>(
      'emits [] when nothing is added',
      build: ReviewRevocationCodeBloc.new,
      expect: () => [],
      verify: (bloc) => expect(bloc.state, const ReviewRevocationCodeInitial()),
    );

    blocTest<ReviewRevocationCodeBloc, ReviewRevocationCodeState>(
      'emits [ReviewRevocationCodeProvidePin] when ReviewRevocationCodeRequested is added',
      build: ReviewRevocationCodeBloc.new,
      act: (bloc) => bloc.add(const ReviewRevocationCodeRequested()),
      expect: () => [const ReviewRevocationCodeProvidePin()],
    );

    blocTest<ReviewRevocationCodeBloc, ReviewRevocationCodeState>(
      'emits [ReviewRevocationCodeSuccess] when ReviewRevocationCodeLoaded is added',
      build: ReviewRevocationCodeBloc.new,
      act: (bloc) => bloc.add(const ReviewRevocationCodeLoaded('test-code')),
      expect: () => [const ReviewRevocationCodeSuccess('test-code')],
    );

    blocTest<ReviewRevocationCodeBloc, ReviewRevocationCodeState>(
      'emits [ReviewRevocationCodeInitial] when ReviewRevocationCodeRestartFlow is added from [ReviewRevocationCodeProvidePin]',
      build: ReviewRevocationCodeBloc.new,
      seed: () => const ReviewRevocationCodeProvidePin(),
      act: (bloc) => bloc.add(const ReviewRevocationCodeRestartFlow()),
      expect: () => [const ReviewRevocationCodeInitial()],
    );

    blocTest<ReviewRevocationCodeBloc, ReviewRevocationCodeState>(
      'emits [ReviewRevocationCodeInitial] when ReviewRevocationCodeRestartFlow is added from [ReviewRevocationCodeSuccess]',
      build: ReviewRevocationCodeBloc.new,
      seed: () => const ReviewRevocationCodeSuccess('test-code'),
      act: (bloc) => bloc.add(const ReviewRevocationCodeRestartFlow()),
      expect: () => [const ReviewRevocationCodeInitial()],
    );
  });
}
