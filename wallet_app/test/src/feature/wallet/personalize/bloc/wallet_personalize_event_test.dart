import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/wallet/personalize/bloc/wallet_personalize_bloc.dart';

void main() {
  group('WalletPersonalizeEvent', () {
    group('WalletPersonalizeLoginWithDigidClicked', () {
      test('supports value comparisons', () {
        expect(
          WalletPersonalizeLoginWithDigidClicked(),
          WalletPersonalizeLoginWithDigidClicked(),
        );
      });
    });

    group('WalletPersonalizeUpdateState', () {
      test('supports value comparisons', () {
        const state = WalletPersonalizeInitial();
        expect(
          const WalletPersonalizeUpdateState(state),
          const WalletPersonalizeUpdateState(state),
        );
      });

      test('is different when state is different', () {
        expect(
          const WalletPersonalizeUpdateState(WalletPersonalizeInitial()),
          isNot(const WalletPersonalizeUpdateState(WalletPersonalizeLoadingIssuanceUrl())),
        );
      });
    });

    group('WalletPersonalizeContinuePidIssuance', () {
      test('supports value comparisons', () {
        expect(
          const WalletPersonalizeContinuePidIssuance('url'),
          const WalletPersonalizeContinuePidIssuance('url'),
        );
      });

      test('is different when authUrl is different', () {
        expect(
          const WalletPersonalizeContinuePidIssuance('url1'),
          isNot(const WalletPersonalizeContinuePidIssuance('url2')),
        );
      });
    });

    group('WalletPersonalizeLoginWithDigidSucceeded', () {
      test('supports value comparisons', () {
        final attributes = [
          DataAttribute.untranslated(
            key: 'key',
            label: 'label',
            value: const StringValue('value'),
          ),
        ];
        expect(
          WalletPersonalizeLoginWithDigidSucceeded(attributes),
          WalletPersonalizeLoginWithDigidSucceeded(attributes),
        );
      });

      test('is different when attributes are different', () {
        expect(
          WalletPersonalizeLoginWithDigidSucceeded([
            DataAttribute.untranslated(
              key: 'key',
              label: 'label',
              value: const StringValue('value'),
            ),
          ]),
          isNot(
            WalletPersonalizeLoginWithDigidSucceeded([
              DataAttribute.untranslated(
                key: 'key',
                label: 'label',
                value: const StringValue('different value'),
              ),
            ]),
          ),
        );
      });
    });

    group('WalletPersonalizeLoginWithDigidFailed', () {
      test('supports value comparisons', () {
        const error = GenericError('error', sourceError: 'source');
        expect(
          const WalletPersonalizeLoginWithDigidFailed(error: error, cancelledByUser: true),
          const WalletPersonalizeLoginWithDigidFailed(error: error, cancelledByUser: true),
        );
      });

      test('is different when error is different', () {
        expect(
          const WalletPersonalizeLoginWithDigidFailed(
            error: GenericError('error1', sourceError: 'source'),
            cancelledByUser: true,
          ),
          isNot(
            const WalletPersonalizeLoginWithDigidFailed(
              error: GenericError('error2', sourceError: 'source'),
              cancelledByUser: true,
            ),
          ),
        );
      });

      test('is different when cancelledByUser is different', () {
        const error = GenericError('error', sourceError: 'source');
        expect(
          const WalletPersonalizeLoginWithDigidFailed(error: error, cancelledByUser: true),
          isNot(const WalletPersonalizeLoginWithDigidFailed(error: error, cancelledByUser: false)),
        );
      });
    });

    group('WalletPersonalizeAcceptPidFailed', () {
      test('supports value comparisons', () {
        const error = GenericError('error', sourceError: 'source');
        expect(
          const WalletPersonalizeAcceptPidFailed(error: error),
          const WalletPersonalizeAcceptPidFailed(error: error),
        );
      });
    });

    group('WalletPersonalizeOfferingAccepted', () {
      test('supports value comparisons', () {
        final attributes = [
          DataAttribute.untranslated(
            key: 'key',
            label: 'label',
            value: const StringValue('value'),
          ),
        ];
        expect(
          WalletPersonalizeOfferingAccepted(attributes),
          WalletPersonalizeOfferingAccepted(attributes),
        );
      });
    });

    group('WalletPersonalizeOfferingRejected', () {
      test('supports value comparisons', () {
        expect(
          WalletPersonalizeOfferingRejected(),
          WalletPersonalizeOfferingRejected(),
        );
      });
    });

    group('WalletPersonalizeRetryPressed', () {
      test('supports value comparisons', () {
        expect(
          WalletPersonalizeRetryPressed(),
          WalletPersonalizeRetryPressed(),
        );
      });
    });

    group('WalletPersonalizeBackPressed', () {
      test('supports value comparisons', () {
        expect(
          WalletPersonalizeBackPressed(),
          WalletPersonalizeBackPressed(),
        );
      });
    });

    group('WalletPersonalizePinConfirmed', () {
      test('supports value comparisons', () {
        expect(
          const WalletPersonalizePinConfirmed(TransferState.available),
          const WalletPersonalizePinConfirmed(TransferState.available),
        );
      });

      test('is different when transferState is different', () {
        expect(
          const WalletPersonalizePinConfirmed(TransferState.available),
          isNot(const WalletPersonalizePinConfirmed(TransferState.unavailable)),
        );
      });
    });
  });
}
