import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/feature/renew_pid/bloc/renew_pid_bloc.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../mocks/wallet_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockGetPidRenewalUrlUseCase getPidRenewalUrlUseCase;
  late MockContinuePidIssuanceUseCase continuePidIssuanceUseCase;
  late MockCancelPidIssuanceUseCase cancelPidIssuanceUseCase;
  late MockGetPidCardsUseCase getPidCardsUseCase;

  setUp(() {
    // Provide dummies
    provideDummy<Result<List<Attribute>>>(const Result.success([]));
    provideDummy<Result<List<WalletCard>>>(const Result.success([]));
    provideDummy<Result<bool>>(const Result.success(true));
    // Create mock usecases
    getPidRenewalUrlUseCase = MockGetPidRenewalUrlUseCase();
    continuePidIssuanceUseCase = MockContinuePidIssuanceUseCase();
    cancelPidIssuanceUseCase = MockCancelPidIssuanceUseCase();
    getPidCardsUseCase = MockGetPidCardsUseCase();
  });

  group('RenewPidBloc', () {
    test('initial state is RenewPidInitial when continueFromDigiD is false', () {
      final bloc = RenewPidBloc(
        getPidRenewalUrlUseCase,
        continuePidIssuanceUseCase,
        cancelPidIssuanceUseCase,
        getPidCardsUseCase,
        continueFromDigiD: false,
      );

      expect(bloc.state, const RenewPidInitial());
    });

    test('initial state is RenewPidVerifyingDigidAuthentication when continueFromDigiD is true', () {
      final bloc = RenewPidBloc(
        getPidRenewalUrlUseCase,
        continuePidIssuanceUseCase,
        cancelPidIssuanceUseCase,
        getPidCardsUseCase,
        continueFromDigiD: true,
      );

      expect(bloc.state, const RenewPidVerifyingDigidAuthentication());
    });

    blocTest<RenewPidBloc, RenewPidState>(
      'happy path: login with DigiD, confirm attributes, confirm pin, succeed',
      build: () {
        when(getPidRenewalUrlUseCase.invoke()).thenAnswer((_) async => const Result.success('mock_auth_url'));
        when(continuePidIssuanceUseCase.invoke('mock_auth_url'))
            .thenAnswer((_) async => Result.success([WalletMockData.textDataAttribute]));
        when(getPidCardsUseCase.invoke()).thenAnswer((_) async => Result.success([WalletMockData.card]));
        when(cancelPidIssuanceUseCase.invoke()).thenAnswer((_) async => const Result.success(true));
        return RenewPidBloc(
          getPidRenewalUrlUseCase,
          continuePidIssuanceUseCase,
          cancelPidIssuanceUseCase,
          getPidCardsUseCase,
          continueFromDigiD: false,
        );
      },
      act: (bloc) async {
        bloc.add(const RenewPidLoginWithDigidClicked());
        await Future.delayed(const Duration(milliseconds: 10));
        bloc.add(const RenewPidContinuePidRenewal('mock_auth_url'));
        await Future.delayed(const Duration(milliseconds: 10));
        bloc.add(RenewPidAttributesConfirmed([WalletMockData.textDataAttribute]));
        await Future.delayed(const Duration(milliseconds: 10));
        bloc.add(RenewPidPinConfirmed());
      },
      expect: () => [
        const RenewPidLoadingDigidUrl(),
        const RenewPidAwaitingDigidAuthentication('mock_auth_url'),
        const RenewPidVerifyingDigidAuthentication(),
        RenewPidCheckData(availableAttributes: [WalletMockData.textDataAttribute]),
        RenewPidConfirmPin([WalletMockData.textDataAttribute]),
        const RenewPidUpdatingCards(),
        RenewPidSuccess([WalletMockData.card]),
      ],
    );

    blocTest<RenewPidBloc, RenewPidState>(
      'emits RenewPidInitial when RenewPidAttributesRejected is added',
      build: () => RenewPidBloc(
        getPidRenewalUrlUseCase,
        continuePidIssuanceUseCase,
        cancelPidIssuanceUseCase,
        getPidCardsUseCase,
        continueFromDigiD: false,
      ),
      seed: () => RenewPidCheckData(availableAttributes: [WalletMockData.textDataAttribute]),
      act: (bloc) => bloc.add(const RenewPidAttributesRejected()),
      expect: () => [const RenewPidInitial(didGoBack: true)],
    );

    blocTest<RenewPidBloc, RenewPidState>(
      'handles back navigation from RenewPidConfirmPin',
      build: () => RenewPidBloc(
        getPidRenewalUrlUseCase,
        continuePidIssuanceUseCase,
        cancelPidIssuanceUseCase,
        getPidCardsUseCase,
        continueFromDigiD: false,
      ),
      seed: () => RenewPidConfirmPin([WalletMockData.textDataAttribute, WalletMockData.textDataAttribute]),
      act: (bloc) => bloc.add(const RenewPidBackPressed()),
      expect: () => [
        RenewPidCheckData(
          availableAttributes: [WalletMockData.textDataAttribute, WalletMockData.textDataAttribute],
          didGoBack: true,
        ),
      ],
    );

    blocTest<RenewPidBloc, RenewPidState>(
      'handles network error when getting DigiD URL',
      build: () {
        when(getPidRenewalUrlUseCase.invoke())
            .thenAnswer((_) async => const Result.error(NetworkError(hasInternet: false, sourceError: 'test')));
        when(cancelPidIssuanceUseCase.invoke()).thenAnswer((_) async => const Result.success(true));
        return RenewPidBloc(
          getPidRenewalUrlUseCase,
          continuePidIssuanceUseCase,
          cancelPidIssuanceUseCase,
          getPidCardsUseCase,
          continueFromDigiD: false,
        );
      },
      act: (bloc) => bloc.add(const RenewPidLoginWithDigidClicked()),
      expect: () => [
        const RenewPidLoadingDigidUrl(),
        isA<RenewPidNetworkError>(),
      ],
    );

    blocTest<RenewPidBloc, RenewPidState>(
      'handles generic error when getting DigiD URL',
      build: () {
        when(getPidRenewalUrlUseCase.invoke())
            .thenAnswer((_) async => const Result.error(GenericError('some error', sourceError: 'ex')));
        when(cancelPidIssuanceUseCase.invoke()).thenAnswer((_) async => const Result.success(true));
        return RenewPidBloc(
          getPidRenewalUrlUseCase,
          continuePidIssuanceUseCase,
          cancelPidIssuanceUseCase,
          getPidCardsUseCase,
          continueFromDigiD: false,
        );
      },
      act: (bloc) => bloc.add(const RenewPidLoginWithDigidClicked()),
      expect: () => [
        const RenewPidLoadingDigidUrl(),
        isA<RenewPidGenericError>(),
      ],
    );

    blocTest<RenewPidBloc, RenewPidState>(
      'handles RenewPidPinConfirmationFailed by exposing the contained error',
      build: () {
        when(cancelPidIssuanceUseCase.invoke()).thenAnswer((_) async => const Result.success(true));
        return RenewPidBloc(
          getPidRenewalUrlUseCase,
          continuePidIssuanceUseCase,
          cancelPidIssuanceUseCase,
          getPidCardsUseCase,
          continueFromDigiD: false,
        );
      },
      act: (bloc) => bloc.add(const RenewPidPinConfirmationFailed(error: GenericError('fail', sourceError: 'fail'))),
      expect: () => [
        isA<RenewPidGenericError>().having((it) => it.error.sourceError, 'Source error should match', 'fail'),
      ],
    );

    blocTest<RenewPidBloc, RenewPidState>(
      'emits RenewPidDigidLoginCancelled when DigiD reports user cancellation through RedirectUriError',
      build: () {
        when(cancelPidIssuanceUseCase.invoke()).thenAnswer((_) async => const Result.success(true));
        return RenewPidBloc(
          getPidRenewalUrlUseCase,
          continuePidIssuanceUseCase,
          cancelPidIssuanceUseCase,
          getPidCardsUseCase,
          continueFromDigiD: false,
        );
      },
      act: (bloc) => bloc.add(
        const RenewPidLoginWithDigidFailed(
          error: RedirectUriError(redirectError: RedirectError.loginRequired, sourceError: 'mock'),
        ),
      ),
      expect: () => [isA<RenewPidDigidLoginCancelled>()],
    );

    blocTest<RenewPidBloc, RenewPidState>(
      'emits RenewPidStopped on stop',
      build: () => RenewPidBloc(
        getPidRenewalUrlUseCase,
        continuePidIssuanceUseCase,
        cancelPidIssuanceUseCase,
        getPidCardsUseCase,
        continueFromDigiD: false,
      ),
      act: (bloc) => bloc.add(const RenewPidStopPressed()),
      expect: () => [const RenewPidStopped()],
    );

    blocTest<RenewPidBloc, RenewPidState>(
      'retry: RenewPidRetryPressed triggers RenewPidLoginWithDigidClicked flow',
      build: () {
        when(getPidRenewalUrlUseCase.invoke()).thenAnswer((_) async => const Result.success('mock_auth_url'));
        when(cancelPidIssuanceUseCase.invoke()).thenAnswer((_) async => const Result.success(true));
        return RenewPidBloc(
          getPidRenewalUrlUseCase,
          continuePidIssuanceUseCase,
          cancelPidIssuanceUseCase,
          getPidCardsUseCase,
          continueFromDigiD: false,
        );
      },
      act: (bloc) => bloc.add(const RenewPidRetryPressed()),
      expect: () => [const RenewPidLoadingDigidUrl(), const RenewPidAwaitingDigidAuthentication('mock_auth_url')],
    );
  });
}
